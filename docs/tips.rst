===============
Tips And Tricks
===============



Debugging
=========

It's often useful to strace_ or use gdb_ on a process for debugging. Usually
you don't install these tools *inside* the container, but run them outside.
Still sometimes you need to attach before proccess starts. Here is how you do
it::

    $ vagga --wait-for-debugger run_some_command
    Pid 1234, countdown: 10 sec...
    # ... the in another terminal
    $ strace -f -p 1234

Not that unless :ref:`pid1mode<pid1mode>` is ``exec``, the target command will
be started as a child of the process displayed, so you always want to start
strace with ``-f`` flag.

For commands using ``supervise`` you need another form of the command::

    vagga run-bunch-of-processes --debug-process process_name

Only single process might be a subject of debugging. But you can use ``--only``
and ``--exclude`` flags for running many of them.


.. _strace: http://en.wikipedia.org/wiki/Strace
.. _gdb: http://www.gnu.org/software/gdb/
