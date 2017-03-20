.. _capsule_commands:

================
Capsule Commands
================

**This functionality is experimental**. Some details can change in future.

.. versionadded:: 0.7.1

It's generally not recommended to use CapsuleCommand, unless you know what are
you doing.

This kind of command doesn't require container to be built. It operates in
intermediate container that we call **capsule**. Capsule is a container that
provides same level of isolation as normal container but has neither config
nor version, on the other hand it provides tools to build create and start
other containers.

This feature is both: more powerful, as it provides a way to build/run
different containers based on dynamic parameters and even change
``vagga.yaml``. On the other hand it starts with bare shell, and it's your job
to bootstrap needed utilities and do all process supervision.

All the tools officially supported by vagga in capsule are prefixed with
``vagga _capsule``, namely:

* ``vagga _capsule build <container_name>`` -- builds container, similar to
  ``vagga _build <container_name>``
* ``vagga _capsule run <container> <cmd>`` -- runs command in a container.
  Container will be (re)built if required.
* ``vagga _capsule script <url>`` -- fetches a script from the url, caches it
  and runs from cache

There are few limitations of the capsule:

1. All containers must have same uid/gid maps (which is often the case)
2. ``vagga _clean`` doesn't work in capsule yet
3. Volume init commands do not work
4. Supervise commands can't be run in capsule (actually any commands configured
   in yaml can't be run from the inside capsule, but most of them can be
   emulated with ``vagga _capsule run``)
