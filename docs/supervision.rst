.. _supervision:

===========
Supervision
===========

Vagga may supervise multiple processes with single command. This is very
useful for running multiple-component and/or networking systems.


By supervision we mean running multiple processes and watching until all them
exit. Each process is run in it's own container. Even if two processes share
the key named "container", which means they share same root filesystem, they
run in different namespaces, so they don't share ``/tmp``, ``/proc`` and so on.


Supervision Modes
=================

There are three basic modes of operation:

* ``stop-on-failure`` -- stops all processes as soon as any single one is dead
  (default)
* ``wait-all`` -- wait for all processes to finish
* ``restart`` -- always restart dead processes

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

In this mode vagga returns ``1`` if some process is dead before vagga received
``SIGINT`` or ``SIGTERM`` signal. Exit code is ``0`` if one of the two received
by vagga. And an ``128+signal`` code when any other singal was sent to
supervisor (and propagated to other processes).


Wait
----

In ``wait`` mode vagga waits that all processes are exited before shutting
down. If any is dead, it's ok, all other will continue as usual.

This mode is intended for running some batch processing of multiple commands
in multiple containers. All processes are run in parallel, like with other
modes.

.. note:: Depending on ``pid1mode`` of each proccess in each container vagga will
   wait either only for process spawned by vagga (``pid1mode: wait`` or
   ``pidmode: exec``), or for all (including daemonized) processes spawned by
   that command (``pid1mode: wait-all-children``). See :ref:`pid1mode` for
   details.


Restart
-------

This is a supervision mode that most other supervisors obey. If one of the
processes is dead, it will be restarted without messing with other processes.

It's not recommended mode for workstations but may be useful for staging
server (Currenly, we do not recommend running vagga in production at all).

.. note:: The whole container is restarted on process failure, so ``/tmp`` is
   clean, all daemonized processes are killed, etc. See also :ref:`pid1mode`.



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
