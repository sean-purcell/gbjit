macro_rules! gen_binary_enum {
    ( $name:ident, $vt: ty, $o0:ident => $v0:expr, $o1:ident => $v1:expr, ) => {
        #[derive(Debug, Copy, Clone)]
        enum $name {
            $o0,
            $o1,
        }

        impl $name {
            fn val(self) -> $vt {
                match self {
                    $name::$o0 => $v0,
                    $name::$o1 => $v1,
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                $name::$o0
            }
        }

        impl From<bool> for $name {
            fn from(v: bool) -> Self {
                if v {
                    $name::$o1
                } else {
                    $name::$o0
                }
            }
        }

        impl From<$name> for bool {
            fn from(v: $name) -> Self {
                match v {
                    $name::$o0 => false,
                    $name::$o1 => true,
                }
            }
        }
    };
}

macro_rules! write_bitfield {
    { $( $idx:expr => $field:expr, )* } => {
        (
        $(
            to_flag($field.into(), $idx) |
        )*
        0)
    }
}

macro_rules! read_bitfield {
    { $val:expr, $( $idx:expr => $field:expr, )* } => {{
        $(
            $field = from_flag($val, $idx).into();
        )*
    }}
}
