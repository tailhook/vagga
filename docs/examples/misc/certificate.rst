Adding a Custom Certificate
===========================

This is useful if you have self-signed sertificates that you use on local or
stating or corporate resources.

In ubuntu it looks like this:

.. code-block:: yaml

   containers:
     some-container:
       setup:
       - !Ubuntu xenial
       - !Install [ca-certificates]
       - !Download
         url: http://example.com/your_company_root.crt
         path: /usr/local/share/ca-certificates/your_company_root.crt
       - !Sh update-ca-certificates


Important thing here is that ``http://example.com/your_company_root.crt``
should be either on a HTTP (not encrypted) host or have a certificate signed
by a well-known authority (included in ubuntu ``ca-certificates`` package).
