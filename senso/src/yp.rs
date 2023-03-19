use crate::{
    connector::Connector,
    request::emf::{Query, TimeRange},
    response::{
        emf_devices::{EmfFunction, EmfType, EnergyType},
        emf_report_device,
    },
};
use anyhow::{anyhow, bail};
use array2d::Array2D;
use chrono::{NaiveDate, NaiveDateTime};
use cli_table::Table;

/// data for central heating and hotwater with total
#[derive(Debug, Table)]
pub struct YpData {
    pub ts: NaiveDateTime,
    pub ch_hp_y: f64,
    pub ch_hp_p: f64,
    pub ch_bo_p: f64,
    pub ch_yp: f64,
    pub hw_hp_y: f64,
    pub hw_hp_p: f64,
    pub hw_bo_p: f64,
    pub hw_yp: f64,
    pub total_y: f64,
    pub total_p: f64,
    pub total_yp: f64,
}

impl YpData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ts: NaiveDateTime,
        ch_hp_y: f64,
        ch_hp_p: f64,
        ch_bo_p: f64,
        ch_yp: f64,
        hw_hp_y: f64,
        hw_hp_p: f64,
        hw_bo_p: f64,
        hw_yp: f64,
        total_y: f64,
        total_p: f64,
        total_yp: f64,
    ) -> Self {
        Self {
            ts,
            ch_hp_y,
            ch_hp_p,
            ch_bo_p,
            ch_yp,
            hw_hp_y,
            hw_hp_p,
            hw_bo_p,
            hw_yp,
            total_y,
            total_p,
            total_yp,
        }
    }
}

pub fn calc_yp(y: f64, p: f64) -> f64 {
    if p == 0.0 {
        0.0
    } else {
        // round with precision of 4 places after 0.0000
        (((y + p) / p) * 10000.0).round() / 10000.0
    }
}

pub fn build_yp_data_vec(
    dhw: UsageFunctionWeek,
    ch: UsageFunctionWeek,
) -> anyhow::Result<Vec<YpData>> {
    // get data for central heatings
    // boiler and heat pump
    let ch_hp_y_vec: Vec<f64> = ch
        .get_dataset(EmfType::HeatPump, EnergyType::EnvironmentalYield)
        .iter()
        .map(|x| x.value)
        .collect();
    let ch_hp_p_vec: Vec<f64> = ch
        .get_dataset(EmfType::HeatPump, EnergyType::ConsumedElectricalPower)
        .iter()
        .map(|x| x.value)
        .collect();
    let ch_bo_p_vec: Vec<f64> = ch
        .get_dataset(EmfType::Boiler, EnergyType::ConsumedElectricalPower)
        .iter()
        .map(|x| x.value)
        .collect();

    // power usage of central heating
    let ch_p_vec: Vec<f64> = ch_hp_p_vec
        .iter()
        .zip(ch_bo_p_vec.iter())
        .map(|x| *x.0 + *x.1)
        .collect();
    // yp of central heating
    let cp_yp_vec: Vec<f64> = ch_hp_y_vec
        .iter()
        .zip(ch_p_vec.iter())
        .map(|x| calc_yp(*x.0, *x.1))
        .collect();

    // get data for hot water
    // boiler and heat pump
    let hw_hp_y_vec: Vec<f64> = dhw
        .get_dataset(EmfType::HeatPump, EnergyType::EnvironmentalYield)
        .iter()
        .map(|x| x.value)
        .collect();
    let hw_hp_p_vec: Vec<f64> = dhw
        .get_dataset(EmfType::HeatPump, EnergyType::ConsumedElectricalPower)
        .iter()
        .map(|x| x.value)
        .collect();
    let hw_bo_p_vec: Vec<f64> = dhw
        .get_dataset(EmfType::Boiler, EnergyType::ConsumedElectricalPower)
        .iter()
        .map(|x| x.value)
        .collect();

    // power usage of how water
    let hw_p_vec: Vec<f64> = hw_hp_p_vec
        .iter()
        .zip(hw_bo_p_vec.iter())
        .map(|x| x.0 + x.1)
        .collect();
    // yp of hot water
    let hw_yp_vec: Vec<f64> = hw_hp_y_vec
        .iter()
        .zip(hw_p_vec.iter())
        .map(|x| calc_yp(*x.0, *x.1))
        .collect();

    // total
    // yield
    // only from heatpump
    let total_y: Vec<f64> = ch_hp_y_vec
        .iter()
        .zip(hw_hp_y_vec.iter())
        .map(|x| x.0 + x.1)
        .collect();
    // power
    let total_p: Vec<f64> = ch_p_vec
        .iter()
        .zip(hw_p_vec.iter())
        .map(|x| x.0 + x.1)
        .collect();
    // yp
    let total_yp: Vec<f64> = total_y
        .iter()
        .zip(total_p.iter())
        .map(|x| calc_yp(*x.0, *x.1))
        .collect();

    // create matrix from all the vecs
    // order important for result
    let vec2d: Vec<Vec<f64>> = vec![
        ch_hp_y_vec,
        ch_hp_p_vec,
        ch_bo_p_vec,
        cp_yp_vec,
        hw_hp_y_vec,
        hw_hp_p_vec,
        hw_bo_p_vec,
        hw_yp_vec,
        total_y,
        total_p,
        total_yp,
    ];

    let matrix = match Array2D::from_rows(&vec2d) {
        Ok(m) => m,
        Err(_) => bail!("Can't create Array2D from vec2d"),
    };

    // create 2d vec but over columns
    let matrix = matrix.as_columns();

    // create timestamp
    // TODO check if correct
    let mut timestamps: Vec<NaiveDateTime> = Vec::with_capacity(7);

    for day in 0..=6_u8 {
        // SAFTEY weekday enum is defined between 0 and 6
        timestamps.push(
            NaiveDate::from_isoywd_opt(dhw.year, dhw.week_nr, unsafe { std::mem::transmute(day) })
                .ok_or(anyhow!("out-of-range date and/or invalid week number"))?
                .and_hms_opt(0, 0, 0)
                .ok_or(anyhow!(""))?,
        );
    }

    let mut result: Vec<YpData> = Vec::with_capacity(7);

    // order from vec2d
    for (i, data) in matrix.iter().enumerate() {
        result.push(YpData::new(
            timestamps[i],
            data[0],
            data[1],
            data[2],
            data[3],
            data[4],
            data[5],
            data[6],
            data[7],
            data[8],
            data[9],
            data[10],
        ))
    }
    Ok(result)
}

#[derive(Debug)]
pub struct UsageFunctionWeek<'a> {
    function: EmfFunction,
    devices: &'a Vec<(EmfType, &'a str)>,
    year: i32,
    week_nr: u32,
    data: Vec<(EmfType, EnergyType, emf_report_device::Root)>,
}

impl<'a> UsageFunctionWeek<'a> {
    pub fn new(
        function: EmfFunction,
        devices: &'a Vec<(EmfType, &'a str)>,
        year: i32,
        week_nr: u32,
    ) -> Self {
        Self {
            function,
            devices,
            data: Vec::with_capacity(devices.len() * 2),
            year,
            week_nr,
        }
    }

    // CALLS API
    // !! currently only one week is supported
    pub fn retrieve_data(&mut self, conn: &Connector) -> anyhow::Result<()> {
        let start = NaiveDate::from_isoywd_opt(self.year, self.week_nr, chrono::Weekday::Mon)
            .ok_or(anyhow!("out-of-range date and/or invalid week number"))?
            .and_hms_opt(0, 0, 0)
            .ok_or(anyhow!(""))?;

        let q_power = Query::new(
            EnergyType::ConsumedElectricalPower,
            self.function,
            TimeRange::Week,
            start,
            None,
        );

        let q_yield = Query::new(
            EnergyType::EnvironmentalYield,
            self.function,
            TimeRange::Week,
            start,
            None,
        );

        // call api for every device
        for d in self.devices {
            let resp_power = conn.emf_report_device(d.1, &q_power)?;
            self.data
                .push((d.0, EnergyType::ConsumedElectricalPower, resp_power));

            // Boiler has no yield
            if d.0 != EmfType::Boiler {
                let resp_yield = conn.emf_report_device(d.1, &q_yield)?;
                self.data
                    .push((d.0, EnergyType::EnvironmentalYield, resp_yield));
            }
        }

        Ok(())
    }

    pub fn get_dataset(
        &self,
        emf_type: EmfType,
        energy_type: EnergyType,
    ) -> Vec<&emf_report_device::Dataset> {
        let t: &Vec<&emf_report_device::Dataset> = &self
            .data
            .iter()
            .filter_map(|f| {
                if f.0 == emf_type && f.1 == energy_type {
                    // body should always includes one dataset
                    Some(&f.2.body.first()?.dataset)
                } else {
                    None
                }
            })
            .flatten()
            .collect();

        t.to_owned()
    }
}

#[cfg(test)]
mod tests {}
