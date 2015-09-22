.. highlight:: bash

.. _environment:

===========
Environment
===========

There are a few ways to pass environment variables from the runner's
environment into a container.

Firstly, any enviroment variable that starts with ``VAGGAENV_`` will have it's
prefix stripped, and exposed in the container's environment::

    $ VAGGAENV_FOO=BAR vagga _run container printenv FOO
    BAR

The ``-e`` or ``--use-env`` command line option can be used to mark environment
variables from the runner's environment that should be passed to container::

    $ FOO=BAR vagga --use-env=FOO _run container printenv FOO
    BAR

And finally the ``-E``, ``--env`` or ``--environ`` command line option can be
used to assign an environment variable that will be passed to the container::

    $ vagga --environ FOO=BAR _run container printenv FOO
    BAR
