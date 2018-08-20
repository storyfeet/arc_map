//! AMapErr exists to allow ? operations in main code.

use std::sync::mpsc::{SendError,RecvError};
use std::sync::{PoisonError,MutexGuard};
use std::convert::From;


#[derive(Debug)]
pub enum AMapErr {
    SendErr,
    RecvErr,
    PoisonErr,
    NotFound,
}



impl<T> From<SendError<T>> for AMapErr{
    fn from(_:SendError<T>)->Self{
        AMapErr::SendErr
    }
}

impl From<RecvError> for AMapErr{
    fn from(_:RecvError)->Self{
        AMapErr::RecvErr
    }
}

impl<'a,V> From<PoisonError<MutexGuard<'a, V>>> for AMapErr{
    fn from(_:PoisonError<MutexGuard<'a,V>>)->Self{
        AMapErr::PoisonErr
    }
}
