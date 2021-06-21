#!/bin/bash

#SBATCH --time=24:00:00   # walltime
#SBATCH --nodes=1   # number of nodes
#SBATCH --ntasks=1      # limit to one node
#SBATCH --cpus-per-task=24  # number of processor cores (i.e. threads)
#SBATCH --partition=haswell
#SBATCH --mem-per-cpu=2048M   # memory per CPU core
#SBATCH --output=/scratch/ws/0/s8179597-triads/com2526.output
#SBATCH -J "com2526"   # job name
#SBATCH -A p_triads

srun ./target/release/tripolys \
	--data /scratch/ws/0/s8179597-triads/data \
	--nodes 25-26 \
	--polymorphism commutative

scontrol show job "$SLURM_JOB_ID"
