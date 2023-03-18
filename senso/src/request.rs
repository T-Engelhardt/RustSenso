pub mod emf {
    use chrono::{DateTime, Local};
    use strum_macros::AsRefStr;

    use crate::response::emf_devices::{EmfFunction, EnergyType};

    #[derive(Debug, AsRefStr)]
    pub enum TimeRange {
        #[strum(serialize = "WEEK")]
        Week,
    }

    #[derive(Debug)]
    pub struct Query {
        time_range: TimeRange,
        start: String,
        offset: Option<String>, // defaults to 0 in iter
        energy_type: EnergyType,
        function: EmfFunction,
    }

    impl Query {
        /// offset defaults to 0 on None
        /// energy_type and function is emitted if None
        pub fn new(
            energy_type: EnergyType,
            function: EmfFunction,
            time_range: TimeRange,
            start: DateTime<Local>,
            offset: Option<String>,
        ) -> Self {
            Self {
                energy_type,
                function,
                time_range,
                start: format!("{}", start.format("%Y-%m-%d")),
                offset,
            }
        }
    }

    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=0cbc1cd12cfdb201aa036ab7b2192a42
    impl<'a> IntoIterator for &'a Query {
        type Item = (&'a str, &'a str);
        type IntoIter = QueryIntoIterator<'a>;

        fn into_iter(self) -> Self::IntoIter {
            QueryIntoIterator {
                query: self,
                index: 0,
            }
        }
    }

    pub struct QueryIntoIterator<'a> {
        query: &'a Query,
        index: usize,
    }

    pub fn empty_query<'a>() -> Vec<(&'a str, &'a str)> {
        Vec::with_capacity(0)
    }

    impl<'a> Iterator for QueryIntoIterator<'a> {
        type Item = (&'a str, &'a str);
        fn next(self: &mut QueryIntoIterator<'a>) -> Option<(&'a str, &'a str)> {
            let result = match self.index {
                0 => ("timeRange", self.query.time_range.as_ref()),
                1 => ("start", self.query.start.as_ref()),
                2 => ("energyType", self.query.energy_type.as_ref()),
                3 => ("function", self.query.function.as_ref()),
                4 => {
                    if let Some(offset) = &self.query.offset {
                        ("offset", offset.as_ref())
                    } else {
                        ("offset", "0")
                    }
                }
                _ => return None,
            };
            self.index += 1;
            Some(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Local;

    use crate::response::emf_devices::{EmfFunction, EnergyType};

    use super::emf::*;

    #[test]
    fn emf_query() {
        let x = Query::new(
            EnergyType::ConsumedElectricalPower,
            EmfFunction::CentralHeating,
            TimeRange::Week,
            Local::now(),
            None,
        );

        for p in x.into_iter() {
            println!("{:?}", p);
        }
    }
}
