.. default-domain:: vagga

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


.. opt:: storage-dir

    Directory where to put images build by vagga. Usually they are stored in
    ``.vagga`` subdirectory of the project dir. It's mostly useful when the
    ``storage-dir`` points to a directory on a separate partition. Path may
    start with ``~/`` which means path is inside the user's home directory.

.. opt:: cache-dir

    Directory where to put cache files during the build. This is used to speed
    up the build process. By default cache is put into ``.vagga/.cache`` in
    project directory but this setting allows to have cache directory shared
    between multiple projects. Path may start with ``~/`` which means path is
    inside the user's home directory.

.. opt:: site-settings

    The mapping of project paths to settings for this specific
    project.

    Example:

    .. code-block:: yaml

       site-settings:
         /home/myuser/myproject:
           cache-dir: /home/myuser/.cache/myproject

.. opt:: proxy-env-vars

    Enable forwarding for proxy environment variables. Default ``true``.
    Environment variables currently that this setting influence currently:
    ``http_proxy``, ``https_proxy``, ``ftp_proxy``, ``all_proxy``,
    ``no_proxy``.

.. opt:: external-volumes

   A mapping of volume names to the directories inside the host file system.

   .. note:: The directories must exist even if unused in any ``vagga.yaml``.

   For example, here is how you might export home:

   .. code-block:: yaml

      external-volumes:
        home: /home/user

   Then in `vagga.yaml` you use it as follows (prepend with `/volumes`):

   .. code-block:: yaml

      volumes:
        /root: !BindRW /volumes/home

   See :ref:`volumes` for more info about defining mount points.

   .. warning::

      1. Usage of volume is usually a subject for filesystem permissions. I.e.
         your user becomes `root` inside the container, and many system users
         are not mapped (not present) in container at all. This means that
         mounting `/var/lib/mysql` or something like that is useless, unless
         you chown the directory

      2. Any vagga project may use the volume if it's defined in global
         config. You may specify the volume in :opt:`site-settings` if you
         care about security (and you should).

.. opt:: push-image-script

   A script to use for uploading a container image when you run
   `vagga _push_image`.

   To push image using webdav::

       push-image-script: "curl -T ${image_path} \
           http://example.org/${container_name}.${short_hash}.tar.xz"

   To push image using `scp` utility (SFTP protocol)::

       push-image-script: "scp ${image_path} \
          user@example.org:/target/path/${container_name}.${short_hash}.tar.xz"

   The FTP(s) (for example, using `lftp` utility) or S3 (using `s3cmd`) are
   also valid choices.

   .. note:: This is that rare case where command is run by vagga in your host
      filesystem. This allows you to use your credentials in home directory,
      and ssh-agent's socket. But also this means that utility to upload
      images must be installed in host system.

   Variables:

   container_name
       The name of the container as declared in `vagga.yaml`

   short_hash
       The short hash of container setup. This is the same hash that is used
       to detect whether container configuration changed and is needed to
       be rebuilt. And the same hash used in directory name `.vagga/.roots`.

.. opt:: auto-apply-sysctl

    Set sysctls required by command. We do our best to only apply "safe"
    sysctls by vagga automatically. Still it may exhaust resources of your
    system, so use this option on your own risk.

    We apply settings with ``sudo -k`` which means it will prompt for password
    each time setting is tuned (probably only after system reboot).

    Settings currently exists:

    ============================= ============================= ===============
    Key in vagga.yaml             Sysctl Name                   Hardcoded Limit
    ============================= ============================= ===============
    :opt:`expect-inotify-limit`   fs.inotify.max_user_watches   524288
    ============================= ============================= ===============

All project-local settings are also allowed here.


Project-Local Settings
======================

Project-local settings may be in the project dir in:

 * ``.vagga.settings.yaml``
 * ``.vagga/settings.yaml``

All project-local settings are also allowed in global config.

While settings can potentially be checked-in to version control it's advised
not to do so.

.. opt:: version-check

    If set to ``true`` (default) vagga will check if the container that is
    already built is up to date with config. If set to ``false`` vagga will
    use any container with same name already built. It's only useful for
    scripts for performance reasons or if you don't have internet and
    containers are not too outdated.

.. opt:: ubuntu-mirror

    Set to your preferred ubuntu mirror. Default is currently a special
    url ``mirror://mirrors.ubuntu.com/mirrors.txt`` which choses local mirror
    for you. But it sometimes fails. Therefore we reserve an option to change
    the default later.

    The best value for this settings is probably
    ``http://<COUNTRY_CODE>.archive.ubuntu.com/ubuntu/``.


.. opt:: alpine-mirror

    Set to your preferred alpine mirror. By default it's the random one is
    picked from the list.

    .. note:: Alpine package manager is used not only for building
       :step:`Alpine` distribution, but also internally for fetching tools that
       are outside of the container filesystem (for example to fetch ``git``
       for :step:`Git` or :step:`GitInstall` command(s))

.. opt:: build-lock-wait

    By default (``build-lock-wait: false``) vagga stops current command and
    prints a message when some other process have already started to build the
    image. When this flag is set to ``true`` vagga will wait instead. This
    is mostly useful for CI systems.

.. opt:: environ

    The mapping, that overrides environment variables set in container or command.
