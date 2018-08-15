.. _supervision:

===========
Supervision
===========

Vagga may supervise multiple processes with single command. This is very
useful for running multiple-component and/or networking systems.


By supervision we mean running multiple processes and watching until all of them
exit. Each process is run in it's own container. Even if two processes share
the key named "container", which means they share same root filesystem, they
run in different namespaces, so they don't share ``/tmp``, ``/proc`` and so on.


Supervision Modes
=================

There are two basic modes of operation:

* ``stop-on-failure`` -- stops all processes as soon as any single one is dead
  (default)
* ``wait-all-successful`` -- waits until all successful processes finish

In any mode of operation supervisor itself never exits until all the children
are dead. Even when you kill supervisor with ``kill -9`` or ``kill -KILL`` all
children will be killed with ``-KILL`` signal too. I.e. with the help of
namespaces and good old ``PR_SET_PDEATHSIG`` we ensure that no process left
when supervisor killed, no one is reparented to ``init``, all traces of running
containers are cleared. Seriously. It's very often a problem with many other
ways to run things on development machine.


Stop on Failure
---------------

It's not coincidence that ``stop-on-failure`` mode is default. It's very
useful mode of operation for running on development machine.

Let me show an example:

.. code-block:: yaml

  commands:
    run_full_app: !Supervise
      mode: stop-on-failure
      children:
        web: !Command
          container: python
          run: "python manage.py runserver"
        celery: !Command
          container: python
          run: "python manage.py celery worker"

Imagine this is a web application written in python (``web`` process), with
a work queue (``celery``), which runs some long-running tasks in background.

When you start both processes ``vagga run_full_app``, often many log messages
with various levels of severity appear, so it's easy to miss something. Imagine
you missed that celery is not started (or dead shortly after start). You go to
the web app do some testing, start some background task, and wait for it to
finish. After waiting for a while, you start suspect that something is wrong.
But celery is dead long ago, so skimming over recent logs doesn't show up
anything. Then you look at processes: "Oh, crap, there is no celery". This is
time-wasting.

With ``stop-on-failure`` you'll notice that some service is down immediately.

In this mode vagga returns exit code of first process exited. And an
``128+signal`` code when any other singal was sent to supervisor (and
propagated to other processes).


Wait All Successful
-------------------

In ``wait-all-successful`` mode vagga works same as in ``stop-on-failure``
mode, except processes that exit with exit code ``0`` (which is known as
sucessful error code) do not trigger failure condition, so other processes
continue to work. If any process exits on signal or with non-zero exit code
"failure" mode is switched on and vagga exits the same as in
``stop-on-failure`` mode.

This mode is intended for running some batch processing of multiple commands
in multiple containers. All processes are run in parallel, like with other
modes.

In this mode vagga returns exit code zero if all processes exited successfully
and exit code of the first failing process (or ``128+signal`` if it was dead
by signal) otherwise.


Tips
====


Restarting a Subset Of Processes
--------------------------------

Sometimes you may work only on one component, and don't want to restart the
whole bunch of processes to test just one thing. You may run two supervisors,
in different tabs of a terminal. E.g:

.. code-block:: bash

    # run everything, except the web process we are debugging
    $ vagga run_full_app --exclude web
    # then in another tab
    $ vagga run_full_app --only web

Then you can restart ``web`` many times, without restarting everything.
