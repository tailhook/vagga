================
Vagga Exit Codes
================

As usually vagga runs some command in a container, it's usually returns status
code from that utility. So vagga uses exit codes >= 120, similarly to shell:

* 127 -- no such vagga command
* 126 -- config not found
* 121 -- miscelaneous error
* 122 -- command-line arguments error
* 128 + x -- process exited on signal x

Note, command inside vagga may potentially return one of the codes above too.
