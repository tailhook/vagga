===============
Firefox Browser
===============

To run firefox, or any other GUI application, there are some extra steps
involved to setup a display.

The ``/tmp/.X11-unix/`` director should be mounted in the container. This
can be accomplished by making it available to vagga under the name ``X11``
by writing the following lines in your global configuration ``~/.vagga.yaml``::

    external-volumes:
      X11: /tmp/.X11-unix/

Next, you can use the following ``vagga.yaml`` file to setup the actual 
configuration. 

.. literalinclude:: ../../../examples/firefox/vagga.yaml
   :language: yaml

When calling vagga, remember to export the ``DISPLAY`` 
environment variable and set the ``HOME`` to be ``/tmp``  to allow firefox 
a place to write the user profile::

    vagga -eDISPLAY -EHOME=/tmp firefox

To prevent DBUS-related errors also export the ``DBUS_SESSION_BUS_ADDRESS``
environmental variable::

   vagga -eDISPLAY -EHOME=/tmp -eDBUS_SESSION_BUS_ADDRESS firefox


WebGL Support
-------------

To enable webgl support further steps are necessary to install the 
drivers inside the container, and that ultimately depends on your video card
model.

To setup the proprietary nvidia drivers, download the driver from the 
`NVIDIA website <http://www.nvidia.ca/Download/index.aspx?lang=en-us>`_ and
use the following ``vagga.yaml``:

.. literalinclude:: ../../../examples/firefox/vagga_webgl_nvidia.yaml
   :language: yaml
 
For intel videocards use the following ``vagga.yaml`` (this includes also 
chromium and java plugins):

.. literalinclude:: ../../../examples/firefox/vagga_webgl_intel.yaml
   :language: yaml
