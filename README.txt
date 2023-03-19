tcping
=======

usage:

    tcping <hostname> [<port>]
    tcping <hsotname:port>
    tcping [arguments] <hostname>


arguments:

    -c, --count
    -t, --forever
    -i, --interval
    -w, --timeout
    -4, --ipv4
    -6, --ipv6

examples:

    tcping localhost
    tcping localhost 80
    tcping localhost:80
    tcping -c 10 localhost
    tcping -t localhost
    tcping 127.0.0.1
    tcping 127.0.0.1 80
    tcping 127.0.0.1:80
    tcping [::1]
    tcping [::1]:80
    tcping ::1
    ~~tcping ::1:80~~ (!won't work)

build:

    cargo build --release