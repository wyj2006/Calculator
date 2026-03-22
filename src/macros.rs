#[macro_export]
macro_rules! forward_impl_binop {
    (impl $(<$($generic:ident),+>)? $imp:ident for $res:ty, $method:ident $(, where $($bounds:tt)+)?) => {
        impl $(<$($generic),+>)? $imp<$res> for $res
            $(where $($bounds)+)?
        {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(&self, &other)
            }
        }

        impl $(<$($generic),+>)? $imp<$res> for &$res
            $(where $($bounds)*)?
        {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(self, &other)
            }
        }

        impl $(<$($generic),+>)? $imp<&$res> for $res
            $(where $($bounds)*)?
        {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(&self, other)
            }
        }
    };

    (impl $(<$($generic:ident),+>)? $imp:ident<$res2:ty> for $res:ty, $method:ident $(, where $($bounds:tt)+)?) => {
        impl $(<$($generic),+>)? $imp<$res2> for $res
            $(where $($bounds)+)?
        {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res2) -> $res {
                $imp::$method(&self, &other)
            }
        }

        impl $(<$($generic),+>)? $imp<$res2> for &$res
            $(where $($bounds)*)?
        {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res2) -> $res {
                $imp::$method(self, &other)
            }
        }

        impl $(<$($generic),+>)? $imp<&$res2> for $res
            $(where $($bounds)*)?
        {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$res2) -> $res {
                $imp::$method(&self, other)
            }
        }
    };
}
