import json

d = json.load(open(r'E:\python\MC\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
r = d['noise_router']
fd = r['final_density']


def walk(v, depth=0):
    if isinstance(v, dict):
        t = v.get('type', '?')
        if t == 'minecraft:range_choice':
            inp = v['input'] if isinstance(v['input'], str) else 'obj'
            print('  '*depth + 'RC input=' + str(inp), 'min=' + str(v['min_inclusive']), 'max=' + str(v['max_exclusive']))
            print('  '*depth + '  IN: ' + json.dumps(v['when_in_range'])[:90])
            print('  '*depth + '  OUT: ' + json.dumps(v['when_out_of_range'])[:90])
        else:
            for k, val in v.items():
                if k == 'type':
                    continue
                if isinstance(val, (dict, list)):
                    walk(val, depth+1)
                else:
                    print('  '*depth + str(k) + ': ' + str(val))
    elif isinstance(v, list):
        for x in v:
            walk(x, depth)


print('=== initial_density ===')
walk(r['initial_density_without_jaggedness'])
print('=== final_density.argument1 (squeeze 内) ===')
walk(fd['argument1']['argument'])
