.. highlight:: yaml

===========
Build Steps
===========

This is work in progress reference of build steps. See :ref:`build_commands`
for help until this document is done.

All of the following build steps may be used as an item in :opt:`setup`
setting.


Container Bootstrap
===================

Command that can be used to bootstrap a container (i.e. may work on top
of empty container):

* :step:`Alpine`
* :step:`Ubuntu`
* :step:`UbuntuRelease`
* :step:`SubConfig`
* :step:`Container`
* :step:`Tar`

Ubuntu Commands
===============

.. step:: Ubuntu

   Simple and straightforward way to install Ubuntu LTS release.

   Example::

       setup:
       - !Ubuntu trusty

   The value is single string having the codename of release ``trusty`` or
   ``precise`` known to work at the time of writing.

   The Ubuntu LTS images are updated on daily basis. But vagga downloads and
   caches the image. To update the image that was downloaded by vagga you need
   to clean the cache.

.. step:: UbuntuRelease

.. step:: UbuntuRepo

.. step:: UbuntuUniverse


Alpine Commands
===============

.. step:: Alpine


Distribution Commands
=====================

These commands work for any linux distributions as long as distribution is
detected by vagga. Latter basically means you used :step:`Alpine`,
:step:`Ubuntu`, :step:`UbuntuRelease` in container config (or in parent
config if you use :step:`SubConfig` or :step:`Container`)

.. step:: Install

.. step:: BuildDeps


Generic Commands
================

.. step:: Sh

.. step:: Cmd

.. step:: Tar

.. step:: TarInstall

.. step:: Git

.. step:: GitInstall



Files and Directories
=====================

.. step:: Text

.. step:: Remove

.. step:: EnsureDir

.. step:: EmptyDir

.. step:: CacheDirs


Meta Data
=========

.. step:: Env

.. step:: Depends


Sub-Containers
==============

.. step:: Container

.. step:: SubConfig


Node.JS Commands
================

.. step:: NpmInstall


Python Commands
===============

.. step:: PipConfig

.. step:: Py2Install

.. step:: Py2Requirements

.. step:: Py3Install

.. step:: Py3Requirements

