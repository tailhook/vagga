.. _settings:

========
Settings
========


Global Settings
===============


Settings are searched for in one of the following files:

 * ``$HOME/.config/vagga/settings.yaml``
 * ``$HOME/.vagga/settings.yaml``
 * ``$HOME/.vagga.yaml``


Supported settings:


``storage-dir``
    Directory where to put images build by vagga. Usually they are stored in
    ``.vagga`` subdirectory of the project dir. It's mostly useful when the
    ``storage-dir`` points to a directory on a separate partition.

``cache-dir``
    Directory where to put cache files during the build. This is used to speed
    up the build process. By default cache is put into ``.vagga/.cache`` in
    project directory but this setting allows to have cache directory shared
    between multiple projects.

``site-settings``
    (experimental) The mapping of project paths to settings for this specific
    project.

All project-local settings are also allowed here.


Project-Local Settings
======================

Project-local settings may be in the project dir in::

 * ``.vagga.settings.yaml``
 * ``.vagga/settings.yaml``

All project-local settings are also allowed in global config.

While settings can potentially be checked-in to version control it's advised
not to do so.

``version-check``
    If set to ``true`` (default) vagga will check if the container that is
    already built is up to date with config. If set to ``false`` vagga will
    use any container with same name already built. It's only useful for
    scripts for performance reasons or if you don't have internet and
    containers are not too outdated.

``ubuntu-mirror``
    Set to your preferred ubuntu mirror. By default it's
    ``mirror://mirrors.ubuntu.com/mirrors.txt`` which means mirror will be
    determined automatically. Note that it's different from default in ubuntu
    itself where ``http://archive.ubuntu.com/ubuntu/`` is the default.
