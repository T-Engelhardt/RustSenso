use crate::{
    connector::Connector,
    request::emf::{Query, TimeRange},
    response::{
        emf_devices::{EmfFunction, EmfType, EnergyType},
        emf_report_device,
    },
};
use anyhow::anyhow;
use chrono::{NaiveDate, NaiveDateTime};

pub struct Usage<'a> {
    usage_vec: Vec<UsageFunction<'a>>,
}

impl<'a> Usage<'a> {
    pub fn new(usage_vec: Vec<UsageFunction<'a>>) -> Self {
        Self { usage_vec }
    }

    pub fn get_yield(&self) {}

    pub fn get_power(&self) {}

    pub fn get_yp(&self) {}

    pub fn get_timestamp(&self) {}
}

#[derive(Debug)]
pub struct UsageFunction<'a> {
    function: EmfFunction,
    devices: &'a Vec<(EmfType, &'a str)>,
    year: i32,
    week_nr: u32,
    timestamps: Vec<NaiveDateTime>,
    data: Vec<(EmfType, EnergyType, emf_report_device::Root)>,
    y_total: Vec<f64>,
    p_total: Vec<f64>,
    yp: Vec<f64>,
}

impl<'a> UsageFunction<'a> {
    // !! currently only one week is supported
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
            timestamps: Vec::with_capacity(7),
            year,
            week_nr,
            y_total: Vec::with_capacity(7),
            p_total: Vec::with_capacity(7),
            yp: Vec::with_capacity(7),
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

        // generate timestamps from week nr and year
        for day in 0..=6_u8 {
            // SAFTEY weekday enum is defined between 0 and 6
            self.timestamps.push(
                NaiveDate::from_isoywd_opt(self.year, self.week_nr, unsafe {
                    std::mem::transmute(day)
                })
                .ok_or(anyhow!("out-of-range date and/or invalid week number"))?
                .and_hms_opt(0, 0, 0)
                .ok_or(anyhow!(""))?,
            );
        }

        // calculate yield and power for this Funktion (DHW, CH) => depends on what is set in ::new()
        // iter over dataset per day
        // body only includes on item in vec
        for day in 0..=6 {
            let mut y = 0.0;
            let mut p = 0.0;

            for d in &self.data {
                match d.1 {
                    EnergyType::EnvironmentalYield => {
                        y += d.2.body.first().ok_or(anyhow!("Empty Body"))?.dataset[day].value
                    }
                    EnergyType::ConsumedElectricalPower => {
                        p += d.2.body.first().ok_or(anyhow!("Empty Body"))?.dataset[day].value
                    }
                }
            }
            self.y_total.push(y);
            self.p_total.push(p);
        }

        // calculate y+p/p
        // AZ = Heizwärme (kWh/a) / Strom (kWh/a)
        // Heizwärme Gesamtewärme power + yield
        self.yp = self
            .y_total
            .iter()
            .zip(self.p_total.iter())
            .map(|x| (x.0 + x.1) / x.1)
            .collect();

        Ok(())
    }

    pub fn get_yield(&self, emf_type: EmfType) {}

    pub fn get_power(&self, emf_type: EmfType) {}

    pub fn get_yp(&self) -> &Vec<f64> {
        &self.yp
    }

    pub fn get_timestamp(&self) -> &Vec<NaiveDateTime> {
        &self.timestamps
    }
}

#[cfg(test)]
mod tests {}
