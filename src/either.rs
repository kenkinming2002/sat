#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Iterator for Either<L, R>
    where L: Iterator,
          R: Iterator<Item = L::Item>
{
    type Item = L::Item;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Left(l)  => l.next(),
            Self::Right(r) => r.next(),
        }
    }
}


