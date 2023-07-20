#!/usr/bin/env python3

from itertools import combinations
from os import system, environ
import argparse

def flatten(l):
	return [
		item for sublist in l
			for item in sublist
	]

def main():
	is_ci = bool(environ.get("GITHUB_ACTIONS"))

	p = argparse.ArgumentParser(description='Test all combinations of flags', formatter_class=argparse.ArgumentDefaultsHelpFormatter)

	p.add_argument('-d', '--do', action='store_true', help='actually run cargo check')
	p.add_argument('-v', '--verbose', action='store_true', help='increase verbosity')
	p.add_argument('-e', '--examples', action='store_true', help='run for examples')
	p.add_argument('-f', '--fail', action='store_true', help='Exit on fail')
	p.add_argument('crate', choices=['roblib', 'roblib-server', 'roblib-client'], help='The crate to test')
	p.add_argument('features', nargs='+', help='The features to test')

	config = p.parse_args()

	feature_combinations = flatten([
		combinations(config.features, i + 1)
			for i in range(len(config.features))
	]) + [()]

	for (i, f) in enumerate(feature_combinations):
		cmd = f'cargo clippy -p {config.crate}'

		if len(f) != 0:
			cmd += f' --features \'{" ".join(f)}\''

		if config.examples:
			cmd += ' --examples'

		if is_ci:
			print('##[group]', end='')
		print(f'{i + 1}/{len(feature_combinations)} {cmd}')

		if config.do:
			exit_code = system(cmd)
			if exit_code != 0 and config.fail:
				exit(1)
			print('\n\n')

		if is_ci:
			print('##[endgroup]')

if __name__ == '__main__':
	main()
