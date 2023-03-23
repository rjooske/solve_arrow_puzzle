pub(crate) trait Enumerate2dTrait<I: Iterator>: Iterator {
    fn enumerate_2d(self, width: usize) -> Enumerate2d<I>;
}

impl<I: Iterator> Enumerate2dTrait<I> for I {
    fn enumerate_2d(self, width: usize) -> Enumerate2d<I> {
        Enumerate2d::new(self, width)
    }
}

pub(crate) struct Enumerate2d<I>
where
    I: Iterator,
{
    iter: I,
    count: usize,
    width: usize,
}

impl<I> Iterator for Enumerate2d<I>
where
    I: Iterator,
{
    type Item = ((usize, usize), <I as Iterator>::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let a = self.iter.next()?;
        let x = self.count % self.width;
        let y = self.count / self.width;
        self.count += 1;
        Some(((x, y), a))
    }
}

impl<I: Iterator> Enumerate2d<I> {
    fn new(iter: I, width: usize) -> Self {
        Self {
            iter,
            count: 0,
            width,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Enumerate2dTrait;

    #[test]
    fn from_3x1_to_1x3() {
        let mut iter = "abc".chars().enumerate_2d(1);
        assert_eq!(iter.next().unwrap(), ((0, 0), 'a'));
        assert_eq!(iter.next().unwrap(), ((0, 1), 'b'));
        assert_eq!(iter.next().unwrap(), ((0, 2), 'c'));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn from_9x1_to_3x3() {
        let mut iter = "wunderbar".chars().enumerate_2d(3);
        assert_eq!(iter.next().unwrap(), ((0, 0), 'w'));
        assert_eq!(iter.next().unwrap(), ((1, 0), 'u'));
        assert_eq!(iter.next().unwrap(), ((2, 0), 'n'));
        assert_eq!(iter.next().unwrap(), ((0, 1), 'd'));
        assert_eq!(iter.next().unwrap(), ((1, 1), 'e'));
        assert_eq!(iter.next().unwrap(), ((2, 1), 'r'));
        assert_eq!(iter.next().unwrap(), ((0, 2), 'b'));
        assert_eq!(iter.next().unwrap(), ((1, 2), 'a'));
        assert_eq!(iter.next().unwrap(), ((2, 2), 'r'));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn from_5x1_to_incomplete_2x3() {
        let mut iter = "HELLO".chars().enumerate_2d(2);
        assert_eq!(iter.next().unwrap(), ((0, 0), 'H'));
        assert_eq!(iter.next().unwrap(), ((1, 0), 'E'));
        assert_eq!(iter.next().unwrap(), ((0, 1), 'L'));
        assert_eq!(iter.next().unwrap(), ((1, 1), 'L'));
        assert_eq!(iter.next().unwrap(), ((0, 2), 'O'));
        assert_eq!(iter.next(), None);
    }
}
