=====================================
Network Tolerance Testing (and Nginx)
=====================================

Somewhat tiny example of the network tolerance testing code is contained
in the following example:

.. literalinclude:: ../../../examples/flaky_network/vagga.yaml
   :language: yaml


.. _nginx:

This example also includes almost a smallest possible nginx configuration:

.. literalinclude:: ../../../examples/flaky_network/nginx.conf
   :language: yaml


.. note::

    The nginx spits the following message just after start::

        nginx: [alert] could not open error log file: open() "/var/log/nginx/error.log" failed (30: Read-only file system)

    It's fine, we can't change this directory as it's hardcoded into the
    source.  While we can mount :volume:`Tmpfs` volume into
    ``/var/log/nginx`` we don't have to, as all other messages are actually
    logged into the ``stderr`` as configured. So this is just annoying and
    useless warning that is safe to ignore.

