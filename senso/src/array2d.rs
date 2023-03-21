use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Array2DError {
    #[error("Input data would overflow 2D Array.")]
    InsertOverflow,
    #[error("2DArray full")]
    Full,
}

/// 2D Array
///
/// Best way to create 2D Array with provided macro
/// ```
/// use senso::array2d;
/// let a = array2d![vec![0, 1], vec![0, 1]].unwrap();
///
/// // Choose how to iterate over data.
/// for x in &a.over_columns() {
///     // do x
/// }
/// for x in &a.over_rows() {
///     // do x
/// }
/// ```
///
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

    /// Inserts iterator over rows sequentially
    ///
    /// # Examples
    /// ```
    /// use senso::array2d::Array2D;
    /// let mut a = Array2D::<i32>::new(2, 2);
    /// // don't forget to check Result for Error
    /// let _ = a.insert_over_row(vec![0, 1, 2, 3].into_iter());
    /// ```
    pub fn insert_over_row(
        &mut self,
        iter: impl ExactSizeIterator<Item = T>,
    ) -> Result<(), Array2DError> {
        if self.data.len() >= self.row_max * self.col_max {
            return Err(Array2DError::Full);
        }
        if self.data.len() + iter.len() > self.row_max * self.col_max {
            return Err(Array2DError::InsertOverflow);
        }
        self.data.extend(iter);
        Ok(())
    }

    /// iterate over columns in 1D
    pub fn over_columns(&self) -> OverColumns<'_, T> {
        OverColumns { array: self }
    }

    /// iterate over rows in 1D
    pub fn over_rows(&self) -> OverRows<'_, T> {
        OverRows { array: self }
    }

    /// returns the set max row length of this 2D Array
    pub fn row_max_len(&self) -> usize {
        self.row_max
    }

    /// returns the set max column length of this 2D Array
    pub fn col_max_len(&self) -> usize {
        self.col_max
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

/// Macro to create 2D Array from x Vectors that are inserted sequentially as row/horizontally.
///
/// First vec determines the row length.
/// Number of vecs determines col length.
///
/// Or manually set row and col length
///
/// # Examples
///
/// ```
/// // Create 2D Array with two vecs
/// // with a row length of 2 and a col length of 3
/// use senso::array2d;
/// let a = array2d![vec![0, 1, 2], vec![0, 1, 2]].unwrap();
/// assert_eq!(a.row_max_len(), 3);
/// assert_eq!(a.col_max_len(), 2);
///
/// // Manually set row and col length
/// let b = array2d![3; 2; vec![0, 1, 2], vec![0], vec![1, 2]].unwrap();
/// assert_eq!(b.row_max_len(), 3);
/// assert_eq!(b.col_max_len(), 2);
/// ```
#[macro_export]
macro_rules! array2d {
    // only one vec
    ($first:expr) => {
        {
            // https://stackoverflow.com/a/69515645
            // use closure to return Result
            (|| {
                let mut result = $crate::array2d::Array2D::new($first.len(), 1usize);
                match result.insert_over_row($first.into_iter()) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }
                Ok(result)
            })()
        }
    };
    // 2d array where first vec determines the row length and number of vecs the col length
    ($first:expr, $( $x:expr ),* ) => {
        {
            // https://stackoverflow.com/a/69515645
            // use closure to return Result
            (|| {
                // use count to determine the column length
                let mut result = $crate::array2d::Array2D::new($first.len(),  $crate::count!($($x)*) + 1usize);
                match result.insert_over_row($first.into_iter()) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }
                $(
                    match result.insert_over_row($x.into_iter()) {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    }
                )*
                Ok(result)
            })()
        }
    };
    // manually set row and col
    ($row:expr; $col:expr; $( $x:expr ),* ) => {
        {
            // https://stackoverflow.com/a/69515645
            // use closure to return Result
            (|| {
                let mut result = $crate::array2d::Array2D::new($row, $col);
                $(
                    match result.insert_over_row($x.into_iter()) {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    }
                )*
                Ok(result)
            })()
        }
    };
}

pub struct OverColumns<'a, T> {
    array: &'a Array2D<T>,
}

/// Iterator over columns
pub struct IterColumns<'a, T> {
    array: &'a Array2D<T>,
    count: usize,
}

/// Iterator over columns
impl<'a, T> IntoIterator for &'a OverColumns<'a, T> {
    type Item = &'a T;

    type IntoIter = IterColumns<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterColumns {
            array: self.array,
            count: 0,
        }
    }
}

impl<'a, T> Iterator for IterColumns<'a, T> {
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

    // improve collect
    // exakt size is known
    fn size_hint(&self) -> (usize, Option<usize>) {
        let bound = self.array.data.len() - self.count;
        (bound, Some(bound))
    }
}

/// Iterator over rows
pub struct OverRows<'a, T> {
    array: &'a Array2D<T>,
}

/// Iterator over rows
pub struct IterRows<'a, T> {
    array: &'a Array2D<T>,
    count: usize,
}

impl<'a, T> IntoIterator for &'a OverRows<'a, T> {
    type Item = &'a T;

    type IntoIter = IterRows<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterRows {
            array: self.array,
            count: 0,
        }
    }
}

impl<'a, T> Iterator for IterRows<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.array.data.len() <= self.count {
            None
        } else {
            let i = self.count;
            self.count += 1;
            Some(&self.array.data[i])
        }
    }

    // improve collect
    // exakt size is known
    fn size_hint(&self) -> (usize, Option<usize>) {
        let bound = self.array.data.len() - self.count;
        (bound, Some(bound))
    }
}

#[cfg(test)]
mod tests {
    use crate::array2d::Array2DError;

    #[test]
    fn array2d_test() {
        // manually set row and col length
        let mut x = array2d![3; 2; vec![0, 1, 2], vec![0, 1], vec![2]].unwrap();
        assert_eq!(
            vec![0, 1, 2, 0, 1, 2],
            x.over_rows()
                .into_iter()
                .map(|f| f.clone())
                .collect::<Vec<i32>>()
        );

        // this should fail with InsertOverflow
        match array2d![3; 2; vec![0, 1, 2], vec![0, 1], vec![0, 1]] {
            Ok(_) => assert!(false),
            Err(e) => match e {
                Array2DError::InsertOverflow => assert!(true),
                _ => assert!(false),
            },
        }

        // 2d array is full
        match x.insert_over_row(vec![0].into_iter()) {
            Ok(_) => assert!(false),
            Err(e) => match e {
                Array2DError::Full => assert!(true),
                _ => assert!(false),
            },
        }

        // test iterator over columns and rows
        let x = array2d![vec![0, 1, 2], vec![0, 1, 2]].unwrap();

        assert_eq!(
            vec![0, 0, 1, 1, 2, 2],
            x.over_columns()
                .into_iter()
                .map(|f| f.clone())
                .collect::<Vec<i32>>()
        );

        assert_eq!(
            vec![0, 1, 2, 0, 1, 2],
            x.over_rows()
                .into_iter()
                .map(|f| f.clone())
                .collect::<Vec<i32>>()
        );
    }
}
