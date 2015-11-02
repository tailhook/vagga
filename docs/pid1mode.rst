.. _pid1mode:

==========================
What's Special With Pid 1?
==========================


The first process started by the linux kernel gets PID 1. Similarly when new
PID namespace is created first process started in that namespace gets PID 1
(the PID as seen by the processes in that namespace, in the parent namespace it
gets assigned other PID).

The process with PID 1 differs from the other processes in the following ways:

1. When the process with pid 1 die for any reason, all other processes are
   killed with ``KILL`` signal
2. When any process having children dies for any reason, its children are
   reparented to process with PID 1
3. Many signals which have default action of ``Term`` do not have one for PID 1.

At a glance, first issue looks like the most annoying. But in practice
the most inconvenient one is the last one. For development purposes it
effectively means you can't stop process by sending ``SIGTERM`` or ``SIGINT``,
if process have not installed a signal handler.

At the end of the day, all above means most processes that were not explicitly
designed to run as PID 1 (which are all applications except supervisors), do
not run well. Vagga fixes that by not running process as PID 1.

.. admonition:: Outdated

   The following text is outdated. Vagga doesn't support any pid modes since
   version 0.2.0. This may be fixed in future. We consider this as mostly
   useless feature for development purposes. If you have a good use case please
   `let us know <https://github.com/tailhook/vagga/issues/86>`_.

In fact there are three modes of operation of PID 1 supported by vagga (set by
:opt:`pid1mode`).

* ``wait`` -- (default) run command (usually it gets PID 2) and wait until it
  exits
* ``wait-all-children`` -- run command, then wait all processes in namespace to
  finish
* ``exec`` -- run the command as PID 1, useful only if command itself is
  process supervisor like upstart_, systemd_ or supervisord_

Note that in ``wait`` and ``exec`` modes, when you kill vagga itself with a
signal, it will propagate the signal to the command itself. In
``wait-all-children`` mode, signal will be propagated to all processes in the
container (even if it's some supplementary command run as a child of some
intermediary process). This is rarely the problem.


.. _upstart: http://upstart.ubuntu.com
.. _systemd: http://www.freedesktop.org/wiki/Software/systemd/
.. _supervisord: http://supervisord.org
