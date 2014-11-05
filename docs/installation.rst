============
Installation
============


Generic Installation
====================


Build-time dependencies:

* Rust_ compiler 0.12 (``rustc``)
* ``make`` (probably gnu variant)
* ``git`` (when installing from git source)

Run-time dependencies (basically none):

* ``glibc``


.. note:: Vagga uses linux_ namespaces, so works on linux system only.


Process is as simple as following::

    git submodule update --init
    make
    sudo make install


.. _Rust: http://rust.org
.. _linux: http://kernel.org


Ubuntu
======

Installing ``rustc``::

    sudo apt-add-repository ppa:hansjorg/rust
    sudo apt-get update
    sudo apt-get install rust-0.12

Building vagga::

    git clone git://github.com/tailhook/vagga
    cd vagga
    git submodule update --init
    make

Installing::

    sudo make install PREFIX=/usr


