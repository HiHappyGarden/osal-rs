use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::any::Any;
use core::fmt::Debug;
use crate::Error::{Std, Type};
use crate::ErrorType::*;

pub const WAIT_FOREVER: u64 = 0xFFFF_FFFF_FFFF_FFFF;
pub(crate) const USECS_PER_SEC: u64 = 1_000_000;
pub(crate) const NSECS_PER_SEC: u64 = 1_000_000_000;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(PartialEq)]
pub enum Error {
    Std(i32, &'static str),
    Type(ErrorType, &'static str),
}

impl Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Std(code, msg) => write!(f, "Error::Std({}, {})", code, msg),
            Type(err_type, msg) => write!(f, "Error::Type({}, {})", err_type.code(), msg),
        }
    }
}

pub type ThreadFunc = dyn Fn(Option<Arc<dyn Any + Send + Sync>>) -> Result<Arc<dyn Any + Send + Sync>> + Send + Sync + 'static;

#[repr(i32)]
#[derive(PartialEq, Clone, Copy)]
pub enum ErrorType {
    Invalid = -1,
    OsEno =  0, /* No error */
    OsEperm =  1, /* Operation not permitted */
    OsEnoent =  2, /* No such file or directory */
    OsEsrch =  3, /* No such process */
    OsEintr =  4, /* Interrupted system call */
    OsEio =  5, /* I/O error */
    OsEnxio =  6, /* No such device or address */
    OsE2big =  7, /* Argument list too long */
    OsEnoexec =  8, /* Exec format error */
    OsEbadf =  9, /* Bad file number */
    OsEchild = 10, /* No child processes */
    OsEagain = 11, /* Try again */
    OsEnomem = 12, /* Out of memory */
    OsEacces = 13, /* Permission denied */
    OsEfault = 14, /* Bad address */
    OsEnotblk = 15, /* Block device required */
    OsEbusy = 16, /* Device or resource busy */
    OsEexist = 17, /* File exists */
    OsExdev = 18, /* Cross-device link */
    OsEnodev = 19, /* No such device */
    OsEnotdir = 20, /* Not a directory */
    OsEisdir = 21, /* Is a directory */
    OsEinval = 22, /* Invalid argument */
    OsEnfile = 23, /* File table overflow */
    OsEmfile = 24, /* Too many open files */
    OsEnotty = 25, /* Not a typewriter */
    OsEtxtbsy = 26, /* Text file busy */
    OsEfbig = 27, /* File too large */
    OsEnospc = 28, /* No space left on device */
    OsEspipe = 29, /* Illegal seek */
    OsErofs = 30, /* Read-only file system */
    OsEmlink = 31, /* Too many links */
    OsEpipe = 32, /* Broken pipe */
    OsEdom = 33, /* Math argument out of domain of func */
    OsErange = 34, /* Math result not representable */
    OsEdeadlk = 35, /* Resource deadlock would occur */
    OsEnametoolong = 36, /* File name too long */
    OsEnolck = 37, /* No record locks available */

    /*
    * This error code is special: arch syscall entry code will return
    * -ENOSYS if users try to call a syscall that doesn't exist.  To keep
    * failures of syscalls that really do exist distinguishable from
    * failures due to attempts to use a nonexistent syscall, syscall
    * implementations should refrain from returning -ENOSYS.
    */
    OsEnosys = 38, /* Invalid system call number */

    OsEnotempty = 39, /* Directory not empty */
    OsEloop = 40, /* Too many symbolic links encountered */
    OsEwouldblock = 41, /* Operation would block */
    OsEnomsg = 42, /* No message of desired type */
    OsEidrm = 43, /* Identifier removed */
    OsEchrng = 44, /* Channel number out of range */
    OsEl2nsync = 45, /* Level 2 not synchronized */
    OsEl3hlt = 46, /* Level 3 halted */
    OsEl3rst = 47, /* Level 3 reset */
    OsElnrng = 48, /* Link number out of range */
    OsEunatch = 49, /* Protocol driver not attached */
    OsEnocsi = 50, /* No CSI structure available */
    OsEl2hlt = 51, /* Level 2 halted */
    OsEbade = 52, /* Invalid exchange */
    OsEbadr = 53, /* Invalid request descriptor */
    OsExfull = 54, /* Exchange full */
    OsEnoano = 55, /* No anode */
    OsEbadrqc = 56, /* Invalid request code */
    OsEbadslt = 57, /* Invalid slot */

    // OsEdeadlock = OsEdeadlk,

    OsEbfont = 59, /* Bad font file format */
    OsEnostr = 60, /* Device not a stream */
    OsEnodata = 61, /* No data available */
    OsEtime = 62, /* Timer expired */
    OsEnosr = 63, /* Out of streams resources */
    OsEnonet = 64, /* Machine is not on the network */
    OsEnopkg = 65, /* Package not installed */
    OsEremote = 66, /* Object is remote */
    OsEnolink = 67, /* Link has been severed */
    OsEadv = 68, /* Advertise error */
    OsEsrmnt = 69, /* Srmount error */
    OsEcomm = 70, /* Communication error on send */
    OsEproto = 71, /* Protocol error */
    OsEmultihop = 72, /* Multihop attempted */
    OsEdotdot = 73, /* RFS specific error */
    OsEbadmsg = 74, /* Not a data message */
    OsEoverflow = 75, /* Value too large for defined data type */
    OsEnotuniq = 76, /* Name not unique on network */
    OsEbadfd = 77, /* File descriptor in bad state */
    OsEremchg = 78, /* Remote address changed */
    OsElibacc = 79, /* Can not access a needed shared library */
    OsElibbad = 80, /* Accessing a corrupted shared library */
    OsElibscn = 81, /* .lib section in a.out corrupted */
    OsElibmax = 82, /* Attempting to link in too many shared libraries */
    OsElibexec = 83, /* Cannot exec a shared library directly */
    OsEilseq = 84, /* Illegal byte sequence */
    OsErestart = 85, /* Interrupted system call should be restarted */
    OsEstrpipe = 86, /* Streams pipe error */
    OsEusers = 87, /* Too many users */
    OsEnotsock = 88, /* Socket operation on non-socket */
    OsEdestaddrreq = 89, /* Destination address required */
    OsEmsgsize = 90, /* Message too long */
    OsEprototype = 91, /* Protocol wrong type for socket */
    OsEnoprotoopt = 92, /* Protocol not available */
    OsEprotonosupport = 93, /* Protocol not supported */
    OsEsocktnosupport = 94, /* Socket type not supported */
    OsEopnotsupp = 95, /* Operation not supported on transport endpoint */
    OsEpfnosupport = 96, /* Protocol family not supported */
    OsEafnosupport = 97, /* Address family not supported by protocol */
    OsEaddrinuse = 98, /* Address already in use */
    OsEaddrnotavail = 99, /* Cannot assign requested address */
    OsEnetdown = 100, /* Network is down */
    OsEnetunreach = 101, /* Network is unreachable */
    OsEnetreset = 102, /* Network dropped connection because of reset */
    OsEconnaborted = 103, /* Software caused connection abort */
    OsEconnreset = 104, /* Connection reset by peer */
    OsEnobufs = 105, /* No buffer space available */
    OsEisconn = 106, /* Transport endpoint is already connected */
    OsEnotconn = 107, /* Transport endpoint is not connected */
    OsEshutdown = 108, /* Cannot send after transport endpoint shutdown */
    OsEtoomanyrefs = 109, /* Too many references: cannot splice */
    OsEtimedout = 110, /* Connection timed out */
    OsEconnrefused = 111, /* Connection refused */
    OsEhostdown = 112, /* Host is down */
    OsEhostunreach = 113, /* No route to host */
    OsEalready = 114, /* Operation already in progress */
    OsEinprogress = 115, /* Operation now in progress */
    OsEstale = 116, /* Stale file handle */
    OsEuclean = 117, /* Structure needs cleaning */
    OsEnotnam = 118, /* Not a XENIX named type file */
    OsEnavail = 119, /* No XENIX semaphores available */
    OsEisnam = 120, /* Is a named type file */
    OsEremoteio = 121, /* Remote I/O error */
    OsEdquot = 122, /* Quota exceeded */

    OsEnomedium = 123, /* No medium found */
    OsEmediumtype = 124, /* Wrong medium type */
    OsEcanceled = 125, /* Operation Canceled */
    OsEnokey = 126, /* Required key not available */
    OsEkeyexpired = 127, /* Key has expired */
    OsEkeyrevoked = 128, /* Key has been revoked */
    OsEkeyrejected = 129, /* Key was rejected by service */

    /* for robust mutexes */
    OsEownerdead = 130, /* Owner died */
    OsEnotrecoverable = 131, /* State not recoverable */

    OsErfkill = 132, /* Operation not possible due to RF-kill */

    OsEhwpoison = 133, /* Memory page has hardware error */

    OsOutrng = 135, /* Out of range*/
    OsCasterr = 136, /* Cast error*/
    OsValconv = 137, /* Value conversion error */
    OsErcrc = 138, /* Crc error */
    OsExcmaxval = 139, /* Exceed max values permitted */
    OsGenerr = 140
}

impl ErrorType {

    pub fn new(code: i32) -> ErrorType {
        match code as usize {
            0 => OsEno,
            1 => OsEperm,
            2 => OsEnoent,
            3 => OsEsrch,
            4 => OsEintr,
            5 => OsEio,
            6 => OsEnxio,
            7 => OsE2big,
            8 => OsEnoexec,
            9 => OsEbadf,
            10 => OsEchild,
            11 => OsEagain,
            12 => OsEnomem,
            13 => OsEacces,
            14 => OsEfault,
            15 => OsEnotblk,
            16 => OsEbusy,
            17 => OsEexist,
            18 => OsExdev,
            19 => OsEnodev,
            20 => OsEnotdir,
            21 => OsEisdir,
            22 => OsEinval,
            23 => OsEnfile,
            24 => OsEmfile,
            25 => OsEnotty,
            26 => OsEtxtbsy,
            27 => OsEfbig,
            28 => OsEnospc,
            29 => OsEspipe,
            30 => OsErofs,
            31 => OsEmlink,
            32 => OsEpipe,
            33 => OsEdom,
            34 => OsErange,
            35 => OsEdeadlk,
            36 => OsEnametoolong,
            37 => OsEnolck,
            38 => OsEnosys,
            39 => OsEnotempty,
            40 => OsEloop,
            41 => OsEwouldblock,
            42 => OsEnomsg,
            43 => OsEidrm,
            44 => OsEchrng,
            45 => OsEl2nsync,
            46 => OsEl3hlt,
            47 => OsEl3rst,
            48 => OsElnrng,
            49 => OsEunatch,
            50 => OsEnocsi,
            51 => OsEl2hlt,
            52 => OsEbade,
            53 => OsEbadr,
            54 => OsExfull,
            55 => OsEnoano,
            56 => OsEbadrqc,
            57 => OsEbadslt,
            59 => OsEbfont,
            60 => OsEnostr,
            61 => OsEnodata,
            62 => OsEtime,
            63 => OsEnosr,
            64 => OsEnonet,
            65 => OsEnopkg,
            66 => OsEremote,
            67 => OsEnolink,
            68 => OsEadv,
            69 => OsEsrmnt,
            70 => OsEcomm,
            71 => OsEproto,
            72 => OsEmultihop,
            73 => OsEdotdot,
            74 => OsEbadmsg,
            75 => OsEoverflow,
            76 => OsEnotuniq,
            77 => OsEbadfd,
            78 => OsEremchg,
            79 => OsElibacc,
            80 => OsElibbad,
            81 => OsElibscn,
            82 => OsElibmax,
            83 => OsElibexec,
            84 => OsEilseq,
            85 => OsErestart,
            86 => OsEstrpipe,
            87 => OsEusers,
            88 => OsEnotsock,
            89 => OsEdestaddrreq,
            90 => OsEmsgsize,
            91 => OsEprototype,
            92 => OsEnoprotoopt,
            93 => OsEprotonosupport,
            94 => OsEsocktnosupport,
            95 => OsEopnotsupp,
            96 => OsEpfnosupport,
            97 => OsEafnosupport,
            98 => OsEaddrinuse,
            99 => OsEaddrnotavail,
            100 => OsEnetdown,
            101 => OsEnetunreach,
            102 => OsEnetreset,
            103 => OsEconnaborted,
            104 => OsEconnreset,
            105 => OsEnobufs,
            106 => OsEisconn,
            107 => OsEnotconn,
            108 => OsEshutdown,
            109 => OsEtoomanyrefs,
            110 => OsEtimedout,
            111 => OsEconnrefused,
            112 => OsEhostdown,
            113 => OsEhostunreach,
            114 => OsEalready,
            115 => OsEinprogress,
            116 => OsEstale,
            117 => OsEuclean,
            118 => OsEnotnam,
            119 => OsEnavail,
            120 => OsEisnam,
            121 => OsEremoteio,
            122 => OsEdquot,
            123 => OsEnomedium,
            124 => OsEmediumtype,
            125 => OsEcanceled,
            126 => OsEnokey,
            127 => OsEkeyexpired,
            128 => OsEkeyrevoked,
            129 => OsEkeyrejected,
            130 => OsEownerdead,
            131 => OsEnotrecoverable,
            132 => OsErfkill,
            133 => OsEhwpoison,
            135 => OsOutrng,
            136 => OsCasterr,
            137 => OsValconv,
            138 => OsErcrc,
            139 => OsExcmaxval,
            140 => OsGenerr,
            _ => Invalid,
        }
    }

    pub fn code(&self) -> i32 {
        *self as i32
    }
}

impl ToString for ErrorType {
    fn to_string(&self) -> String {
        match self {
            Invalid => "Invalid".to_string(),
            OsEno => "No error".to_string(),
            OsEperm => "Operation not permitted".to_string(),
            OsEnoent => "No such file or directory".to_string(),
            OsEsrch => "No such process".to_string(),
            OsEintr => "Interrupted system call".to_string(),
            OsEio => "I/O error".to_string(),
            OsEnxio => "No such device or address".to_string(),
            OsE2big => "Argument list too long".to_string(),
            OsEnoexec => "Exec format error".to_string(),
            OsEbadf => "Bad file number".to_string(),
            OsEchild => "No child processes".to_string(),
            OsEagain => "Try again".to_string(),
            OsEnomem => "Out of memory".to_string(),
            OsEacces => "Permission denied".to_string(),
            OsEfault => "Bad address".to_string(),
            OsEnotblk => "Block device required".to_string(),
            OsEbusy => "Device or resource busy".to_string(),
            OsEexist => "File exists".to_string(),
            OsExdev => "Cross-device link".to_string(),
            OsEnodev => "No such device".to_string(),
            OsEnotdir => "Not a directory".to_string(),
            OsEisdir => "Is a directory".to_string(),
            OsEinval => "Invalid argument".to_string(),
            OsEnfile => "File table overflow".to_string(),
            OsEmfile => "Too many open files".to_string(),
            OsEnotty => "Not a typewriter".to_string(),
            OsEtxtbsy => "Text file busy".to_string(),
            OsEfbig => "File too large".to_string(),
            OsEnospc => "No space left on device".to_string(),
            OsEspipe => "Illegal seek".to_string(),
            OsErofs => "Read-only file system".to_string(),
            OsEmlink => "Too many links".to_string(),
            OsEpipe => "Broken pipe".to_string(),
            OsEdom => "Math argument out of domain of func".to_string(),
            OsErange => "Math result not representable".to_string(),
            OsEdeadlk => "Resource deadlock would occur".to_string(),
            OsEnametoolong => "File name too long".to_string(),
            OsEnolck => "No record locks available".to_string(),
            OsEnosys => "Invalid system call number".to_string(),
            OsEnotempty => "Directory not empty".to_string(),
            OsEloop => "Too many symbolic links encountered".to_string(),
            OsEwouldblock => "Operation would block".to_string(),
            OsEnomsg => "No message of desired type".to_string(),
            OsEidrm => "Identifier removed".to_string(),
            OsEchrng => "Channel number out of range".to_string(),
            OsEl2nsync => "Level 2 not synchronized".to_string(),
            OsEl3hlt => "Level 3 halted".to_string(),
            OsEl3rst => "Level 3 reset".to_string(),
            OsElnrng => "Link number out of range".to_string(),
            OsEunatch => "Protocol driver not attached".to_string(),
            OsEnocsi => "No CSI structure available".to_string(),
            OsEl2hlt => "Level 2 halted".to_string(),
            OsEbade => "Invalid exchange".to_string(),
            OsEbadr => "Invalid request descriptor".to_string(),
            OsExfull => "Exchange full".to_string(),
            OsEnoano => "No anode".to_string(),
            OsEbadrqc => "Invalid request code".to_string(),
            OsEbadslt => "Invalid slot".to_string(),
            OsEbfont => "Bad font file format".to_string(),
            OsEnostr => "Device not a stream".to_string(),
            OsEnodata => "No data available".to_string(),
            OsEtime => "Timer expired".to_string(),
            OsEnosr => "Out of streams resources".to_string(),
            OsEnonet => "Machine is not on the network".to_string(),
            OsEnopkg => "Package not installed".to_string(),
            OsEremote => "Object is remote".to_string(),
            OsEnolink => "Link has been severed".to_string(),
            OsEadv => "Advertise error".to_string(),
            OsEsrmnt => "Srmount error".to_string(),
            OsEcomm => "Communication error on send".to_string(),
            OsEproto => "Protocol error".to_string(),
            OsEmultihop => "Multihop attempted".to_string(),
            OsEdotdot => "RFS specific error".to_string(),
            OsEbadmsg => "Not a data message".to_string(),
            OsEoverflow => "Value too large for defined data type".to_string(),
            OsEnotuniq => "Name not unique on network".to_string(),
            OsEbadfd => "File descriptor in bad state".to_string(),
            OsEremchg => "Remote address changed".to_string(),
            OsElibacc => "Can not access a needed shared library".to_string(),
            OsElibbad => "Accessing a corrupted shared library".to_string(),
            OsElibscn => " .lib section in a.out corrupted".to_string(),
            OsElibmax => "Attempting to link in too many shared libraries".to_string(),
            OsElibexec => "Cannot exec a shared library directly".to_string(),
            OsEilseq => "Illegal byte sequence".to_string(),
            OsErestart => "Interrupted system call should be restarted".to_string(),
            OsEstrpipe => "Streams pipe error".to_string(),
            OsEusers => "Too many users".to_string(),
            OsEnotsock => "Socket operation on non-socket".to_string(),
            OsEdestaddrreq => "Destination address required".to_string(),
            OsEmsgsize => "Message too long".to_string(),
            OsEprototype => "Protocol wrong type for socket".to_string(),
            OsEnoprotoopt => "Protocol not available".to_string(),
            OsEprotonosupport => "Protocol not supported".to_string(),
            OsEsocktnosupport => "Socket type not supported".to_string(),
            OsEopnotsupp => "Operation not supported on transport endpoint".to_string(),
            OsEpfnosupport => "Protocol family not supported".to_string(),
            OsEafnosupport => "Address family not supported by protocol".to_string(),
            OsEaddrinuse => "Address already in use".to_string(),
            OsEaddrnotavail => "Cannot assign requested address".to_string(),
            OsEnetdown => "Network is down".to_string(),
            OsEnetunreach => "Network is unreachable".to_string(),
            OsEnetreset => "Network dropped connection because of reset".to_string(),
            OsEconnaborted => "Software caused connection abort".to_string(),
            OsEconnreset => "Connection reset by peer".to_string(),
            OsEnobufs => "No buffer space available".to_string(),
            OsEisconn => "Transport endpoint is already connected".to_string(),
            OsEnotconn => "Transport endpoint is not connected".to_string(),
            OsEshutdown => "Cannot send after transport endpoint shutdown".to_string(),
            OsEtoomanyrefs => "Too many references: cannot splice".to_string(),
            OsEtimedout => "Connection timed out".to_string(),
            OsEconnrefused => "Connection refused".to_string(),
            OsEhostdown => "Host is down".to_string(),
            OsEhostunreach => "No route to host".to_string(),
            OsEalready => "Operation already in progress".to_string(),
            OsEinprogress => "Operation now in progress".to_string(),
            OsEstale => "Stale file handle".to_string(),
            OsEuclean => "Structure needs cleaning".to_string(),
            OsEnotnam => "Not a XENIX named type file".to_string(),
            OsEnavail => "No XENIX semaphores available".to_string(),
            OsEisnam => "Is a named type file".to_string(),
            OsEremoteio => "Remote I/O error".to_string(),
            OsEdquot => "Quota exceeded".to_string(),
            OsEnomedium => "No medium found".to_string(),
            OsEmediumtype => "Wrong medium type".to_string(),
            OsEcanceled => "Operation Canceled".to_string(),
            OsEnokey => "Required key not available".to_string(),
            OsEkeyexpired => "Key has expired".to_string(),
            OsEkeyrevoked => "Key has been revoked".to_string(),
            OsEkeyrejected => "Key was rejected by service".to_string(),
            OsEownerdead => "Owner died".to_string(),
            OsEnotrecoverable => "State not recoverable".to_string(),
            OsErfkill => "Operation not possible due to RF-kill".to_string(),
            OsEhwpoison => "Memory page has hardware error".to_string(),
            OsOutrng => "Out of range".to_string(),
            OsCasterr => "Cast error".to_string(),
            OsValconv => "Value conversion error".to_string(),
            OsErcrc => "Crc error".to_string(),
            OsExcmaxval => "Exceed max values permitted".to_string(),
            OsGenerr => "Generic error".to_string(),
        }
    }
}

impl Debug for ErrorType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} ({})", self.code(), self.to_string())
    }
}

impl Default for ErrorType {
    fn default() -> Self {
        Invalid
    }
}