#!/bin/env python3

import itertools
import sys
import os

def flatten(l):
    return [
		item for sublist in l
			for item in sublist
	]

def main():
	sys.argv.pop(0)
	do_it = len(sys.argv) > 0 and sys.argv[0] == '--do'

	if do_it:
		sys.argv.pop(0)
	else:
		print("--do not provided, running dry")

	default_features = len(sys.argv) == 0

	features = ['roland', 'gpio', 'camloc'] if default_features else sys.argv

	feature_combinations = [
		itertools.combinations(features, i + 1)
			for i in range(len(features))
	]

	feature_combinations = flatten(feature_combinations)
	
	if default_features:
		feature_combinations.insert(0, ('bare',))

	for f in feature_combinations:
		cmd = f'cargo clippy --features "{" ".join(f)}" --examples'
		if do_it:
			print(f'\n\n{cmd}:')
			if os.system(cmd) != 0:
				exit(1)
		else:
			print(cmd)

if __name__ == '__main__':
	main()