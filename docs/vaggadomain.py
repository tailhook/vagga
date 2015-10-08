from sphinxcontrib.domaintools import custom_domain

def setup(app):
    app.add_domain(custom_domain('VaggaOptions',
        name  = 'vagga',
        label = "Vagga Yaml Options",

        elements = dict(
            opt = dict(
                objname      = "Yaml Option",
                indextemplate = "single: %s",
            ),
            cmd = dict(
                objname      = "Vagga Command",
                indextemplate = "single: %s",
            ),
            volume = dict(
                objname      = "Volume Type",
                indextemplate = "pair: %s; Volume Type",
            ),
        )))
