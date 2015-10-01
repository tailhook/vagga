========
Examples
========


By Category
===========

Bellow is a list of sample configs from `vagga/examples
<https://github.com/tailhook/vagga/tree/master/examples>`_. To run any of them
just jump to the folder and run ``vagga``.

Databases
---------

.. toctree::
   :maxdepth: 1

   examples/db/postgres
   examples/db/redis


Miscellaneous
-------------

.. toctree::
   :maxdepth: 1

   examples/misc/travis


Documentation
-------------

.. toctree::
   :maxdepth: 1

   examples/doc/sphinx


Real World Examples
===================

This section contains real-world examples of possibly complex vagga files.
They are represented as external symlinks (github) with a description. Send
a pull request to add your example here.

.. admonition:: First Time User Hint
   :class: admonition hint

   All the examples run in containers and install dependencies in ``.vagga``
   subfolder of project dir. So all that possibly scary dependencies are
   installed automatically and **never touch your host system**. That makes
   it easy to experiment with vagga.

* `Vagga itself`__ -- fairly complex config, includes:

    * *Building* Rust with musl_ libc support
    * Docs using sphinx_ and additional dependencies
    * Running vagga in vagga for tests

    __ https://github.com/tailhook/vagga/blob/master/vagga.yaml
    .. _sphinx: http://sphinx-doc.org/
    .. _musl: http://www.musl-libc.org/

* `Presentation`__ config for simple `impress.js`_ presentation generated
  from `restructured text`_ (``.rst``) files. Includes:

    * Installing hovercraft_ by Pip (Python 3), which generates the HTML files
    * The simple ``serve`` command to serve the presentation on HTTP
    * The ``pdf`` command which generates PDF files using wkhtmltopdf_ and some
      complex bash magic

    __ https://github.com/tailhook/containers-tutorial/blob/master/vagga.yaml
    .. _restructured text: http://sphinx-doc.org/rest.html
    .. _impress.js: https://github.com/impress/impress.js
    .. _hovercraft: http://hovercraft.readthedocs.org/en/latest/presentations.html
    .. _wkhtmltopdf: http://wkhtmltopdf.org/
