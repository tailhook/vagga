
Vagga Commands
==============


build-packages
--------------

:Container: rust-musl

Create an ubuntu (.deb) package using checkinstall in container and tar.gz. Both put into `dist/`


build-packages-testing
----------------------

:Container: rust-musl

Same as build-packages but with debugging info enabled


cargo
-----

:Container: rust-musl

Run arbitrary cargo command


doc
---

:Container: docs

Build vagga documentation


make
----

:Container: rust-musl

Build vagga


make-release
------------

:Container: rust-musl

Build vagga with optimizations


print-env
---------

:Container: docs

no description


test
----

:Container: test

Run self tests


test-internal
-------------

:Container: rust-musl

Run rust tests of vagga

