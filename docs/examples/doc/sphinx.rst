====================
Sphinx Documentation
====================

The simplest way to generate sphinx documentation is to use ``py-sphinx``
package from Alpine linux:

.. literalinclude:: ../../../examples/sphinx_doc/vagga.yaml
   :language: yaml

To start documentation from scratch (if you had no sphinx docs before), run
the following once (and answer the questions)::

    vagga _run doc sphinx-quickstart ./doc



