==============
Selenium Tests
==============

Running selenium with vagga is as easy as anything else.

Setting up the GUI may take some effort because you need a display, but
starting PhantomJS as a driver looks like the following:

.. literalinclude:: ../../../examples/selenium_pytest/vagga.yaml
   :language: yaml

And the test may look like the following:

.. literalinclude:: ../../../examples/selenium_pytest/test.py
   :language: python3

To run the test just type::

    > vagga test

