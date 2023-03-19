pub mod emf {
    use chrono::NaiveDateTime;
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
        /// start uses UTC
        pub fn new(
            energy_type: EnergyType,
            function: EmfFunction,
            time_range: TimeRange,
            start: NaiveDateTime,
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
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    use crate::response::emf_devices::{EmfFunction, EnergyType};

    use super::emf::*;

    #[test]
    fn emf_query() {
        let x = Query::new(
            EnergyType::ConsumedElectricalPower,
            EmfFunction::CentralHeating,
            TimeRange::Week,
            NaiveDateTime::new(
                NaiveDate::from_isoywd_opt(2023, 9, chrono::Weekday::Mon).unwrap(),
                NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            ),
            None,
        );

        for p in x.into_iter() {
            println!("{:?}", p);
        }

        let mut iter = x.into_iter();
        assert_eq!(("timeRange", "WEEK"), iter.next().unwrap());
        assert_eq!(("start", "2023-02-27"), iter.next().unwrap());
        assert_eq!(
            ("energyType", "CONSUMED_ELECTRICAL_POWER"),
            iter.next().unwrap()
        );
        assert_eq!(("function", "CENTRAL_HEATING"), iter.next().unwrap());
        assert_eq!(("offset", "0"), iter.next().unwrap());
        assert!(iter.next().is_none());
    }
}
