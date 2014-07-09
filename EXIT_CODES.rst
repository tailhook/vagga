================
Vagga Exit Codes
================

As usually vagga runs some command in a container, it's usually returns status
code from that utility. So vagga uses exit codes >= 120, similarly to shell:

* 127 -- no such vagga command
* 126 -- config not found
* 121 -- miscelaneous error
* 122 -- command-line arguments error

Note, command inside vagga may return one of the codes above too.
