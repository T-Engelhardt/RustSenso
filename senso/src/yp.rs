use crate::{
    array2d,
    connector::Connector,
    request::emf::{Query, TimeRange},
    response::{
        emf_devices::{EmfDevice, EmfFunction, EnergyType},
        emf_report_device::{self, Dataset},
    },
};
use anyhow::anyhow;
use chrono::{NaiveDate, NaiveDateTime};
use cli_table::Table;
use itertools::Itertools;
use num_traits::cast::FromPrimitive;

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
    // # central heatings
    // heat pump yield
    let ch_hp_y_vec = ch
        .get_dataset(EmfDevice::HeatPump, EnergyType::EnvironmentalYield)
        .map(|x| x.value)
        .collect_vec();
    // heat pump power usage
    let ch_hp_p_vec = ch
        .get_dataset(EmfDevice::HeatPump, EnergyType::ConsumedElectricalPower)
        .map(|x| x.value)
        .collect_vec();
    // boiler power usage
    let ch_bo_p_vec = ch
        .get_dataset(EmfDevice::Boiler, EnergyType::ConsumedElectricalPower)
        .map(|x| x.value)
        .collect_vec();

    // total power usage of central heating
    let ch_p_vec = ch.get_total(EnergyType::ConsumedElectricalPower)?;
    // yp of central heating
    // yield is only generated from the heatpump
    let cp_yp_vec: Vec<f64> = ch_hp_y_vec
        .iter()
        .zip(ch_p_vec.iter())
        .map(|(ch_hp_y, ch_p)| calc_yp(*ch_hp_y, *ch_p))
        .collect_vec();

    // # how water
    // heat pump yield
    let hw_hp_y_vec = dhw
        .get_dataset(EmfDevice::HeatPump, EnergyType::EnvironmentalYield)
        .map(|x| x.value)
        .collect_vec();
    // heat pump power usage
    let hw_hp_p_vec = dhw
        .get_dataset(EmfDevice::HeatPump, EnergyType::ConsumedElectricalPower)
        .map(|x| x.value)
        .collect_vec();
    // boiler power usage
    let hw_bo_p_vec = dhw
        .get_dataset(EmfDevice::Boiler, EnergyType::ConsumedElectricalPower)
        .map(|x| x.value)
        .collect_vec();
    // total power usage of how water
    let hw_p_vec = dhw.get_total(EnergyType::ConsumedElectricalPower)?;
    // yp of hot water
    // yield is only generated from the heatpump
    let hw_yp_vec = hw_hp_y_vec
        .iter()
        .zip(hw_p_vec.iter())
        .map(|(hw_hp_y, hw_p)| calc_yp(*hw_hp_y, *hw_p))
        .collect_vec();

    // # Total
    // yield
    let total_y: Vec<f64> = ch_hp_y_vec
        .iter()
        .zip(hw_hp_y_vec.iter())
        .map(|(ch_hp_y, hw_hp_y)| ch_hp_y + hw_hp_y)
        .collect_vec();
    // power usage
    let total_p: Vec<f64> = ch_p_vec
        .iter()
        .zip(hw_p_vec.iter())
        .map(|(ch_p, hw_p)| ch_p + hw_p)
        .collect_vec();
    // yp
    let total_yp: Vec<f64> = total_y
        .iter()
        .zip(total_p.iter())
        .map(|(y, p)| calc_yp(*y, *p))
        .collect_vec();

    // create matrix from all the vecs
    // order important for result
    let array2d = array2d![
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
        total_yp
    ]?;

    // create timestamp
    // TODO check if correct
    let mut timestamps: Vec<NaiveDateTime> = Vec::with_capacity(7);

    for day in 0..=6_u8 {
        // SAFTEY weekday enum is defined between 0 and 6
        timestamps.push(
            NaiveDate::from_isoywd_opt(
                dhw.year,
                dhw.week_nr,
                chrono::Weekday::from_u8(day).ok_or(anyhow!("Can't convert u8 to Weekday"))?,
            )
            .ok_or(anyhow!("out-of-range date and/or invalid week number"))?
            .and_hms_opt(0, 0, 0)
            .ok_or(anyhow!(""))?,
        );
    }

    let result = array2d
        // iter over colums
        .over_columns()
        .into_iter()
        // create tuple of yp data, order set in array2d
        .tuples::<(_, _, _, _, _, _, _, _, _, _, _)>()
        // add timestamp
        .zip(timestamps.iter())
        // create yp data struct
        .map(
            |(
                (
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
                ),
                ts,
            )| YpData {
                ts: *ts,
                ch_hp_y: *ch_hp_y,
                ch_hp_p: *ch_hp_p,
                ch_bo_p: *ch_bo_p,
                ch_yp: *ch_yp,
                hw_hp_y: *hw_hp_y,
                hw_hp_p: *hw_hp_p,
                hw_bo_p: *hw_bo_p,
                hw_yp: *hw_yp,
                total_y: *total_y,
                total_p: *total_p,
                total_yp: *total_yp,
            },
        )
        .collect();

    Ok(result)
}

/// Power usage and yield for given funtion(HotWater, Heating) and devices(Heatpump, Boiler)
#[derive(Debug)]
pub struct UsageFunctionWeek<'a> {
    function: EmfFunction,
    devices: &'a Vec<(EmfDevice, &'a str)>,
    year: i32,
    week_nr: u32,
    power_usage: Vec<(EmfDevice, emf_report_device::Root)>,
    yield_vec: Vec<(EmfDevice, emf_report_device::Root)>,
}

impl<'a> UsageFunctionWeek<'a> {
    pub fn new(
        function: EmfFunction,
        devices: &'a Vec<(EmfDevice, &'a str)>,
        year: i32,
        week_nr: u32,
    ) -> Self {
        Self {
            function,
            devices,
            power_usage: Vec::with_capacity(devices.len()),
            yield_vec: Vec::with_capacity(devices.len()),
            year,
            week_nr,
        }
    }

    /// Calls remote api for given connector.
    ///
    /// Retrieves data for power usage and yield for given devices and funktion
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
        for (device, device_id) in self.devices {
            let resp_power = conn.emf_report_device(device_id, &q_power)?;
            self.power_usage.push((*device, resp_power));

            // Boiler has no yield
            if *device != EmfDevice::Boiler {
                let resp_yield = conn.emf_report_device(device_id, &q_yield)?;
                self.yield_vec.push((*device, resp_yield));
            }
        }

        Ok(())
    }

    ///
    pub fn get_dataset(
        &self,
        emf_device: EmfDevice,
        energy_type: EnergyType,
    ) -> impl Iterator<Item = &Dataset> {
        let dataset = match energy_type {
            EnergyType::EnvironmentalYield => &self.yield_vec,
            EnergyType::ConsumedElectricalPower => &self.power_usage,
        };

        let dataset = dataset
            .iter()
            // filter for given device and energy
            .filter(move |f| f.0 == emf_device)
            .filter_map(|f| {
                let d = f.1.body.first()?;
                Some(&d.dataset)
            })
            .flatten();

        dataset
    }

    /// Get total power/yield
    pub fn get_total(&self, energy_type: EnergyType) -> anyhow::Result<Vec<f64>> {
        let mut total: Vec<f64> = vec![0.0; 7];
        for (device, _) in self.devices {
            for (i, data) in self.get_dataset(*device, energy_type).enumerate() {
                total[i] = total.get(i).ok_or(anyhow!(
                    "Index of Result(get_total) is out of bound. Dataset is to long for 1 Week."
                ))? + data.value;
            }
        }
        Ok(total)
    }
}

#[cfg(test)]
mod tests {}
