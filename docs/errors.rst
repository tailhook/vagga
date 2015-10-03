======
Errors
======

The document describes errors when running vagga on various systems. The manual
only includes errors which need more detailed explanation and troubleshooting.
Most errors should be self-descriptive.

.. _subuid:

Could not read /etc/subuid or /etc/subgid
-----------------------------------------

The full error might look like::

    ERROR:vagga::container::uidmap: Error reading uidmap: Can't open /etc/subuid: No such file or directory (os error 2)
    WARN:vagga::container::uidmap: Could not read /etc/subuid or /etc/subgid (see http://bit.ly/err_subuid)
    error setting uid/gid mappings: Operation not permitted (os error 1)

This means there is no ``/etc/subuid`` file. It probably means you need to
create one. The recommended contents are following::

    your_user_name:100000:65536

----

You may get another similar error::

    ERROR:vagga::container::uidmap: Error reading uidmap: /etc/subuid:2: Bad syntax: "user:100000:100O"
    WARN:vagga::container::uidmap: Could not read /etc/subuid or /etc/subgid (see http://bit.ly/err_subuid)
    error setting uid/gid mappings: Operation not permitted (os error 1)

This means somebody has edited ``/etc/subuid`` and made an error. Just open
the file (note it's owned by root) and fix the issue (in the example the last
character should be zero, but it's a letter "O").







