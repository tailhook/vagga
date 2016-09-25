RethinkDB
=========

RethinkDB_ is described as:

    RethinkDB is the open-source, scalable database
    that makes building realtime apps dramatically easier.

.. _rethinkdb: https://www.rethinkdb.com/


Because RethinkDB has an Ubuntu package, it's easy to setup:

.. literalinclude:: ../../../examples/rethinkdb/vagga.yaml
   :language: yaml
   :lines: 5-21, 28-33

We also have a configued `example chat`_ application `in the repository`_,
that you may run with alongside with the database itself as follows::

    vagga example-chat

.. _example chat: https://github.com/rethinkdb/rethinkdb-example-nodejs-chat
.. _in the repository: https://github.com/tailhook/vagga/tree/master/examples/rethinkdb
