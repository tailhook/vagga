==========
Travis Gem
==========

The following snippet installs travis gem (into container). For example to
provide github token to `Travis CI`_ (so that it can push to github), you
can run the following::

    $ vagga travis encrypt --repo xxx/yyy --org GH_TOKEN=zzz

The vagga configuration for the command:

.. literalinclude:: ../../../examples/travis_gem/vagga.yaml
   :language: yaml


.. _Travis CI: http://travis-ci.org
