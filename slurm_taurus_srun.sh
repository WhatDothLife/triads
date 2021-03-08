#!/bin/bash

#SBATCH --time=24:00:00   # walltime
#SBATCH --nodes=1   # number of nodes
#SBATCH --ntasks=1      # limit to one node
#SBATCH --cpus-per-task=24  # number of processor cores (i.e. threads)
#SBATCH --partition=haswell
#SBATCH --mem-per-cpu=2048M   # memory per CPU core
#SBATCH --output=/scratch/ws/0/s8179597-triads/jakub_sig.output
#SBATCH -J "jakub_sig"   # job name
#SBATCH -A p_graphen

srun ./target/release/triads \
	--data /scratch/ws/0/s8179597-triads/data \
	--triad 01001111,0110000,101000 \
	--polymorphism siggers

scontrol show job "$SLURM_JOB_ID"
