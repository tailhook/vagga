=============
Release Notes
=============

Vagga 0.2.0
===========


:Release Date: 11.02.2015

This is backwards-incompatible release of vagga. See Upgrading_. The need for
changes in configuration format is dictated by the following:

* Better isolation of build process from host system
* More flexible build steps (i.e. don't fall back to shell scripting for
  everything beyond "install this package")
* Caching for all downloads and packages systems (not only for OS-level
  packages but also for packages installed by pip and npm)
* Deep dependency tracking (in future version we will not only track
  changes of dependencies in ``vagga.yaml`` but also in ``requirements.txt``
  and ``package.json`` or whatever convention exists; it's partially possible
  using Depends_ build step)

More features:

* Built by Rust ``1.0.0-alpha``
* Includes experimental network_ `testing tools`_


There are `some features missing<missing-features>`, but we believe it doesn't
affect a lot of users.


.. _Upgrading: http://vagga.readthedocs.org/en/latest/upgrading.html
.. _missing-features: http://vagga.readthedocs.org/en/latest/upgrading.html#missing-features
.. _Depends: http://vagga.readthedocs.org/en/latest/build_commands.html#depends
.. _network: http://vagga.readthedocs.org/en/latest/network.html
.. _testing tools: https://medium.com/@paulcolomiets/evaluating-mesos-4a08f85473fb
