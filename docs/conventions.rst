.. default-domain:: vagga

===========
Conventions
===========

This document describes the conventions for writing vagga files.  You are free
to use only ones that makes sense for your project.


Motivation
==========

Establishing conventions for vagga file have the following benefits:

* Easy to get into your project for new developers
* Avoid common mistakes when creating vagga file


Command Naming
==============

.. cmd:: run

    To run a project you should just start::

        $ vagga run

    This should obey following rules:

    1. Run all the dependencies: i.e. database, memcache, queues, whatever
    2. Run in host network namespace, so user can access database from host
       without any issues
    3. You shouldn't need to configure anything before running the app, all
       defaults should be out of the box

.. cmd:: test

    To run all automated tests you should start::

        $ vagga test

    This should include:

    1. Run all the test suites that may be run locally
    2. Should not include tests that require external resources
    3. If that's possible, should include ability to run individual tests and
       `--help`
    4. Should run all needed dependencies (databases, caches,..), presumably
       on different ports from ones used for ``vagga run``

    It's expected that exact parameters depend on the underlying project.
    I.e. for python project this would be a thin wrapper around `nosetests`

.. cmd:: test-whatever

    Runs individual test suite. Named ``whatever``. This may be used for
    two purposes:

    1. Test suite requires some external dependencies, say a huge database
       with real-life products from a e-commerce site.
    2. There are multiple test suites with different runners, for example you
       have a `nosetests` runner and `cunit` runner that require different
       command-line to choose individual test to run

    Otherwise it's similar to :cmd:`run` and may contain part of that
    test suite

