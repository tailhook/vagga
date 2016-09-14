.. default-domain:: vagga

.. _commands:

========
Commands
========


Every command under ``commands`` in ``vagga.yaml`` is mapped with a tag
that denotes the command type. The are two command types ``!Command``
and ``!Supervise`` illustrated by the following example:

.. code-block:: yaml

    containers: {ubuntu: ... }
    commands:
      bash: !Command
        description: Run bash shell inside the container
        container: ubuntu
        run: /bin/bash
      download: !Supervise
        description: Download two files simultaneously
        children:
          amd64: !Command
            container: ubuntu
            run: wget https://partner-images.canonical.com/core/xenial/current/ubuntu-xenial-core-cloudimg-amd64-root.tar.gz
          i386: !Command
            container: ubuntu
            run: wget https://partner-images.canonical.com/core/xenial/current/ubuntu-xenial-core-cloudimg-i386-root.tar.gz


Common Parameters
=================

These parameters work for both kinds of commands:


.. opt:: description

   Description that is printed in when vagga is run without arguments

.. opt:: banner

   The message that is printed before running process(es). Useful for
   documenting command behavior.

.. opt:: banner-delay

   The seconds to sleep before printing banner. For example if commands run
   a web service, banner may provide a URL for accessing the service. The
   delay is used so that banner is printed after service startup messages not
   before.  Note that currently vagga sleeps this amount of seconds even
   if service is failed immediately.

.. opt:: epilog

   The message printed after command is run. It's printed only if command
   returned zero exit status. Useful to print further instructions, e.g. to
   display names of build artifacts produced by command.

.. opt:: prerequisites

   The list of commands to run before the command, each time it is started.

   Example:

   .. code-block:: yaml

       commands:
         make:
           container: build
           run: "make prog"
         run:
           container: build
           prerequisites: [make]
           run: "./prog"

   The sequence of running of command with ``prerequesites`` is following:

   1. Container is built if needed for each prerequisite
   2. Container is built if needed for main command
   3. Each prerequisite is run in sequence
   4. Command is started

   If any step fails, neither next step nor the command is run.

   The :opt:`prerequisites` are recursive. If any of the prerequisite has
   prerequisites itself, they will be called. But each named command will be
   run only once. We use topology sort to ensure prerequisite commands are
   started before dependent commands. For cyclic dependencies, we ensure that
   command specified in the command line is run later, otherwise order of
   cyclic dependencies is unspecified.

   The supervise command's ``--only`` and ``--except`` influences neither
   running prerequisites itself nor commands inside the prerequisite if the
   latter happens to be supervise command. But there is a global flag
   ``--no-prerequisites``.

   The :opt:`prerequisites` is not (yet) supported in the any of ``children``
   of a ``!Supervise`` command, but you can write prerequisites for the whole
   command group.

.. opt:: expect-inotify-limit

   Check the sysctl ``fs.inotify.max_user_watches`` and print a warning
   or set it automatically if :opt:`auto-apply-sysctl` is enabled.
   :ref:`More info about max_user_watches <sysctl-max-user-watches>`

.. opt:: options

   This is a docopt_ definition for the options that this command accepts.
   Example:

   .. code-block:: yaml

      commands:
        test: !Supervise
          options: |
            Usage: vagga test [--redis-port=<n>] [options] [<tests>...]

            Options:
              -R, --redis-port <n>  Port to run redis on [default: 6379]
              <tests> ...           Name of the tests to run. By default all
                                    tests are run
          children:
            redis: !Command
              container: redis
              run: |
                redis-server --daemonize no --port "$VAGGAOPT_REDIS_PORT"
            first-line: !Command
              container: busybox
              run: |
                py.test --redis-port "$VAGGAOPT_REDIS_PORT" $VAGGAOPT_TESTS

   As you might have noticed, options are passed in environment variables
   prefixed with ``VAGGAOPT_`` and ``VAGGACLI_`` (see below).
   Your scripts are free to use them however makes sense for your application.

   .. note::

      * You should include ``[options]`` at least in one of the usage examples,
        to have ``-h, --help`` working as well as other built-in options
        (``--only, --except`` in supervise commands)
      * This setting overrides :opt:`accepts-arguments`

   Every argument is translated into two variables:

   * ``VAGGAOPT_ARG`` -- has the raw value of the argument, for boolean flags
     it contains either ``true`` or nothing, for repeatable flags it contains
     a number of occurences
   * ``VAGGACLI_ARG`` -- has a canonical representation of an argument, this
     includes option name and all needed escaping to represent multiple
     command line arguments

   The ``ARG`` is usually a long name of the option if exists, or short
   name otherwise. For positional arguments it's argument name. It's always
   uppercased and has ``-`` replaced with ``_``

   There are few shortcommings of both kinds:

   1. ``VAGGAOPT_`` can't represent list of arguments that can contain
      spaces. So it can't be used for list of file names in the general
      case.
   2. ``VAGGACLI_`` contains escaped versions of arguments so requires
      using ``eval`` to make proper argument list from it

   Some shell patterns using ``VAGGAOPT_``:

   1. To propagate a flag, use either one::

        somecmd ${VAGGAOPT_FLAG:+--flag}
        somecmd $VAGGACLI_FLAG

   2. To optionally pass a value to a command, use either one (note the
      implications of eval in the second command)::

        somecmd ${VAGGAOPT_VALUE:+--value} $VAGGAOPT_VALUE
        eval somecmd $VAGGACLI_VALUE

      To overcome limitations of eval, for example if you need to expand
      ``$(hostname)`` in the command, you can use the following snippet::

        eval printf "'%s\0'" $VAGGACLI_VALUE | xargs -0 somecmd -H$(hostname)

   3. To pass a list of commands each prefixed with a ``--test=``, use either
      one::

        # any shell (but ugly)
        eval printf "'%s\0'" $VAGGACLI_TESTS | sed -z 's/^/--test=/' | xargs -0 somecmd

      ::

        # bash only
        eval "tests=($VAGGACLI_TESTS)"
        somecmd "${tests[@]/#/--test=}"

      (Note for some ``sed`` implementations you need to omit ``-z`` flag)

      This works if you have argument like ``vagga test <tests>...``. However,
      if your vagga command-line is ``vagga test --test=<name>...`` use the
      following instead::

        eval somecmd $VAGGACLI_TEST

.. _docopt: http://docopt.org/


Parameters of `!Command`
========================

.. opt:: container

   The container to run command in.

.. opt:: tags

   The list of tags for this command.
   Tags are used for processes filtering (with ``--only`` and ``--exclude``)
   when running any ``!Supervise`` command.

   Simple example:

   .. code-block:: yaml

      commands:
        run: !Supervise
          children:
            postgres: !Command
              tags: [service]
              run: ...
            redis: !Command
              tags: [service]
              run: ...
            app: !Command
              tags: [app]
              run: ...

   .. code-block:: bash

      $ vagga run --only service  # will start only postgres and redis processes

.. opt:: run

   The command to run. It can be:

   - either a string encompassing a shell command line (which is feeded to
     ``/bin/sh -c``)
   - or a list containing first the full path to the executable to run
     and then possibly static arguments.

.. opt:: work-dir

   The working directory to run in. Path relative to project root. By
   default command is run in the same directory where vagga started (sans
   the it's mounted as ``/work`` so the output of ``pwd`` would seem to be
   different)

.. opt:: accepts-arguments

   Denotes whether command accepts additional arguments. Defaults to:

   - ``false`` for a shell command line (if ``run`` is a string);
   - ``true`` if command is an executable (if ``run`` is a list).

   NB: If command is a shell command line - even if it's composed of
   only one call to an executable -, arguments are given to its
   executing context, not appended to it.

   .. note:: This setting is ignored when :opt:`options` is set.

.. opt:: environ

   The mapping of environment to pass to command. This overrides environment
   specified in container on value by value basis.

.. opt:: volumes

   The mapping of mount points to the definition of volume. Allows to mount
   some additional filesystems inside the container. See :ref:`volumes` for
   more info.

   The volumes defined here override :opt:`volumes` specified in the
   container definition (each volume name is considered separately).

   .. note:: You must create a folder for each volume. See
      :ref:`build_commands` for documentation.

.. opt:: pid1mode

   This denotes what is run as pid 1 in container. It may be ``wait``,
   ``wait-all-children`` or ``exec``. The default ``wait`` is okay for most
   regular processes. See :ref:`pid1mode` for more info.

.. opt:: write-mode

   The parameter specifies how container's base file system is used. By
   default container is immutable (corresponds to the ``read-only`` value of
   the parameter), which means you can only write to the ``/tmp`` or
   to the ``/work`` (which is your project directory).

   Another option is ``transient-hard-link-copy``, which means that whenever
   command is run, create a copy of the container, consisting of hard-links to
   the original files, and remove the container after running command. Should
   be used with care as hard-linking doesn't prevent original files to be
   modified. Still very useful to try package installation in the system. Use
   ``vagga _build --force container_name`` to fix base container if that was
   modified.

.. opt:: user-id

   The user id to run command as. If the ``external-user-id`` is omitted this
   has same effect like using ``sudo -u`` inside container (except it's user
   id instead of user name)

.. _external-user-id:

.. opt:: external-user-id

   **(experimental)** This option allows to map the ``user-id`` as seen by
   command itself to some other user id inside container namespace (the
   namespace which is used to build container). To make things a little less
   confusing, the following two configuration lines:

   .. code-block:: yaml

       user-id: 1
       external-user-id: 0

   Will make your command run as user id 1 visible inside the container
   (which is "daemon" or "bin" depending on distribution). But outside the
   container it will be visible as your user (i.e. user running vagga). Which
   effectively means you can create/modify files in project directory without
   permission errors, but ``tar`` and other commands which have different
   behaviour when running as root would think they are not root (but has
   user id 1)

.. opt:: group-id

   The group id to run command as. Default is ``0``.

.. opt:: supplementary-gids

   The list of group ids of the supplementary groups. By default it's empty
   list.

.. opt:: pass-tcp-socket

   Binds a TCP to the specified address and passes it to the application
   as a file descriptor #3.

   Example:

   .. code-block:: yaml

      nginx:
        container: nginx
        run: nginx
        pass-tcp-socket: 8080
        environ:
          NGINX: "3;" # inform nginx not to listen on its own

   You may specify what to listen to with the following formats:

   * `8080` -- just a port number -- listens on 127.0.0.1
   * `*:8080` -- wildcard pattern for host -- listens on every host
   * `0.0.0.0:8080` -- same as `*:8080`
   * `192.0.2.1:8080` -- listen on specified IPv4 host
   * `[2001:db8::1]:8080` -- listen on specified IPv6 host
   * `localhost:8080` -- resolve a name and listen that host (note: name
     must resolve to a single address)

   This is better then listening by the application itself in the following
   cases:

   1. If you want to test systemd socket activation
   2. If you prepare your application to a powerful supervisor like lithos_
      (lithos can run multiple processes on the same port using the feature)
   3. To declare (document) that your application listens specified port
      (otherwise it may be hidden somewhere deeply in config)
   4. To listen port in the **host** network namespace when applying network
      isolation (as an alternate to :opt:`public-ports`)

   .. _lithos: http://lithos.readthedocs.io


Parameters of `!Supervise`
==========================

.. opt:: mode

   The set of processes to supervise and mode. See :ref:`supervision` for more
   info

.. opt:: children

   A mapping of name to child definition of children to run. All children are
   started simultaneously.

.. opt:: kill-unresponsive-after

   (default `2` seconds) If some process exits (in ``stop-on-failure``
   mode), vagga will send TERM signal to all the other processes. If they don't
   finish in the specified number of seconds, vagga will kill them with KILL
   signal (so they finish without being able to intercept signal
   unconditionally). If you don't like this behavior set the parameter to
   some large value.

.. _isolate-network:

.. opt:: isolate-network

   Run all processes within isolated network namespace. Isolated network will
   have only a loopback device, so processes won't have access to the internet.
   For example, it is possible to run several test suites each start service
   that binds the same port. Also you can run arbitrary command inside isolated
   network using ``--isolate-network`` commandline option.
