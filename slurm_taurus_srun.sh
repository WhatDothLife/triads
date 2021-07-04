#!/bin/bash

#SBATCH --time=48:00:00   # walltime
#SBATCH --nodes=1   # number of nodes
#SBATCH --ntasks=1      # limit to one node
#SBATCH --cpus-per-task=24  # number of processor cores (i.e. threads)
#SBATCH --partition=haswell
#SBATCH --mem-per-cpu=5000M   # memory per CPU core
#SBATCH --output=/scratch/ws/0/s8179597-triads/micdvs.output
#SBATCH -J "micdvs"   # job name
#SBATCH -A p_triads

srun ./target/release/tripolys \
	--data /scratch/ws/0/s8179597-triads/data \
	--triad 10110000,0101111,10011 \
	--polymorphism 3/4wnu

scontrol show job "$SLURM_JOB_ID"
