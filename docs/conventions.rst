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

    The rules for the command:

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
       with real-life products for an e-commerce site.
    2. There are multiple test suites with different runners, for example you
       have a `nosetests` runner and `cunit` runner that require different
       command-line to choose individual test to run

    Otherwise it's similar to :cmd:`run` and may contain part of that
    test suite

.. cmd:: doc

    Builds documentation::

        $ vagga doc
        [.. snip ..]
        --------------------------------------------------------
        Documentation is built under docs/_build/html/index.html

    The important points about the command:

    1. Build HTML documentation
    2. Use :opt:`epilog` to show where the documentation is after build
    3. Use :opt:`work-dir` if your documentation build runs in a subdirectory

    If you don't have HTML documentation at all, just ignore rule #1 and put
    whatever documentation format that makes sense for your project.

    Additional documentation builders (different formats) may be provided by
    other commands. But main ``vagga doc`` command should be enough to validate
    all the docs written before the commit.

    The documentation may be built by the same container that application runs
    or different one, or even just inherit from application's one (useful
    when some of the documentation is extracted from the code).


