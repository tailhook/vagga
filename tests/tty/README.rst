TTY testing
===========

It is hard to test TTY so there are several test cases to check them manually.

* Interactive shell

  Run::

    vagga python

  Test:

  - ``Ctrl+C``: python shell should print ``KeyboardInterrupt``.
  - ``Ctrl+Z``: python shell is stopped (can unfreeze by sending ``SIGCONT``
    signal to the python shell). One day this behavior should be fixed.
  - ``Ctrl+D``: closes python interactive shell.

* Non-interactive commands

  Run::

    vagga slow-counter

    vagga run

  Test:

  - ``Ctrl+Z``: vagga and child processes should be suspended.

    ``jobs`` command should display something like
    ``[1]  + suspended  ../../vagga slow-counter``.

    ``fg`` should continue commands execution.

    Repeat one more time.

  - ``Ctrl+C``: terminates all commands.

* Supervisor with interactive shell

  Run::

    vagga run-interactive

  This command runs redis and python shell processes.

  Test:

  - ``Ctrl+C``: python shell should print ``KeyboardInterrupt``.
  - ``Ctrl+Z``: python shell is stopped (can unfreeze by sending ``SIGCONT``
    signal to the python shell). One day this behavior should be fixed.
  - ``Ctrl+D``: closes python interactive shell, then vagga should kill redis.

* Redirecting stdout to less utility

  Run::

    vagga counter | less

  Test:

  - ``space``: should paginate output.
  - ``Ctrl+Z``: vagga with child processes and less should be suspended.

    ``jobs`` command should display something like::

      [1]  + suspended (signal)  ../../vagga counter |
             suspended           less

    ``fg`` should continue commands execution.

    Repeat one more time.
  - ``Ctrl+C``: vagga should terminate, less should continue working.
  - ``q``: terminates less and vagga.
  - Run ``jobs``: there is no less process in the background.
