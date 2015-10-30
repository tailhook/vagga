import os
from flask import Flask
from flask.ext.sqlalchemy import SQLAlchemy

app = Flask(__name__)
app.config['SQLALCHEMY_DATABASE_URI'] = os.environ.get('DATABASE_URL')
db = SQLAlchemy(app)

@app.route('/')
def hello_world():
    return '; '.join(db.engine.table_names())

if __name__ == '__main__':
    app.run(host='0.0.0.0')
