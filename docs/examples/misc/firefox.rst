===============
Firefox Browser
===============

To run firefox or any other GUI application there are some extra steps
involved to setup a display.

The ``/tmp/.X11-unix/`` directory should be mounted in the container. This
can be accomplished by making it available to vagga under the name ``X11``
by writing the following lines in your `global configuration`_ ``~/.vagga.yaml``::

    external-volumes:
      X11: /tmp/.X11-unix/

.. _global configuration: ../../settings.html#global-settings

Next, you can use the following ``vagga.yaml`` file to setup the actual 
configuration (we redefine the variable ``HOME`` because firefox needs to 
write profile information).

.. literalinclude:: ../../../examples/firefox/vagga.yaml
   :language: yaml

.. note::
   If Firefox is already running on your host system, it will
   connect to it to avoid creating another instance and it
   will use the resources of your host system instead of the container's.

   We pass ``--no-remote`` to tell it to create a new instance
   inside the container, to avoid exposing the host file system.

When calling vagga, remember to export the ``DISPLAY`` 
environment variable::

    vagga -eDISPLAY firefox

To prevent DBUS-related errors also export the ``DBUS_SESSION_BUS_ADDRESS``
environmental variable::

   vagga -eDISPLAY -eDBUS_SESSION_BUS_ADDRESS firefox


WebGL Support
-------------

To enable WebGL support further steps are necessary to install the
drivers inside the container, that depends on your video card model.

To setup the proprietary nvidia drivers, download the driver from the 
`NVIDIA website <http://www.nvidia.ca/Download/index.aspx?lang=en-us>`_ in the 
your working directory and use the following ``vagga.yaml``:

.. literalinclude:: ../../../examples/firefox/vagga_webgl_nvidia.yaml
   :language: yaml
 
For intel video cards use the following ``vagga.yaml`` (this includes also 
chromium and java plugin):

.. literalinclude:: ../../../examples/firefox/vagga_webgl_intel.yaml
   :language: yaml
