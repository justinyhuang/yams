#!/usr/bin/env python
import yaml

with open(r'test/flow.coil.data', 'r+') as file:
    datalist = yaml.load(file, Loader=yaml.FullLoader)
    # set the power led state with the value of power led button
    datalist['db'][10005]['data_value']['value'] = datalist['db'][10006]['data_value']['value']
    file.seek(0)
    documents = yaml.dump(datalist, file)


