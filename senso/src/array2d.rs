use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Array2DError {
    #[error("Data has not the same Row Length")]
    RowLenMismatch,
    #[error("2DArray full")]
    Full,
}

/// Iterates over colums on default.
///
/// Uses self::get_vec() to iterate over rows.
#[derive(Debug)]
pub struct Array2D<T> {
    row_max: usize,
    col_max: usize,
    data: Vec<T>,
}

impl<T> Array2D<T> {
    /// 2DArray with given capacity.
    /// Can't grow larger then given row * col.
    pub fn new(row: usize, col: usize) -> Self {
        Self {
            data: Vec::with_capacity(row * col),
            row_max: row,
            col_max: col,
        }
    }

    /// Inserts rows sequentially
    ///
    /// Data needs to have the length of the given 2darray row length
    pub fn insert_row(&mut self, mut data: Vec<T>) -> Result<(), Array2DError> {
        if self.data.len() >= self.row_max * self.col_max {
            return Err(Array2DError::Full);
        }
        if data.len() != self.row_max {
            return Err(Array2DError::RowLenMismatch);
        }
        self.data.append(&mut data);
        Ok(())
    }

    /// use this if you want to iterate over the rows
    pub fn get_vec(&self) -> &Vec<T> {
        &self.data
    }
}

// MACROS
// for creating a 2dArray from vecs

// helper macro to count number of optional args in macro
#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + $crate::count!($($xs)*));
}

#[macro_export]
macro_rules! array2d {
    // only one vec
    ($first:expr) => {
        {
            // https://stackoverflow.com/a/69515645
            // use closure to return Result
            (|| {
                let mut result = $crate::array2d::Array2D::new($first.len(), 1usize);
                match result.insert_row($first) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }
                Ok(result)
            })()
        }
    };
    // 2d array
    // first vec determines the row length
    ($first:expr, $( $x:expr ),* ) => {
        {
            // https://stackoverflow.com/a/69515645
            // use closure to return Result
            (|| {
                let mut result = $crate::array2d::Array2D::new($first.len(),  $crate::count!($($x)*) + 1usize);
                match result.insert_row($first) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }
                $(
                    match result.insert_row($x) {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    }
                )*
                Ok(result)
            })()
        }
    };
}

/// Iterator over columns
pub struct Iter<'a, T> {
    array: &'a Array2D<T>,
    count: usize,
}

/// Iterator over columns
impl<'a, T> IntoIterator for &'a Array2D<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            array: self,
            count: 0,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.array.data.len() <= self.count {
            None
        } else {
            // get index in array over columns
            let i = (self.count / self.array.col_max)
                + (self.array.row_max * (self.count % self.array.col_max));
            // increment count
            self.count += 1;
            // return ref to T
            Some(&self.array.data[i])
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::array2d::Array2DError;

    #[test]
    fn array2d_test() {
        // this should fail with RowLenMismatch
        match array2d![vec![0, 1, 2], vec![0, 1]] {
            Ok(_) => assert!(false),
            Err(e) => match e {
                Array2DError::RowLenMismatch => assert!(true),
                _ => assert!(false),
            },
        }

        // test iterator over rows
        let mut x = array2d![vec![0, 1, 2], vec![0, 1, 2]].unwrap();

        assert_eq!(
            vec![0, 0, 1, 1, 2, 2],
            x.into_iter().map(|f| f.clone()).collect::<Vec<i32>>()
        );

        // 2d array is full
        match x.insert_row(vec![0]) {
            Ok(_) => assert!(false),
            Err(e) => match e {
                Array2DError::Full => assert!(true),
                _ => assert!(false),
            },
        }
    }
}
