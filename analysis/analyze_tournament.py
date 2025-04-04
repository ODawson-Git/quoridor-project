# --- START OF FILE analysis/analyze_tournament.py ---

import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import numpy as np
import os
import argparse
import sys
import re
import json

# --- Configuration ---
BASE_OUTPUT_DIR = "analysis_outputs"

# --- Helper Function ---
def create_output_directory(csv_filepath):
    """Creates a unique output directory based on the CSV filename."""
    try:
        base_filename = os.path.basename(csv_filepath)
        run_identifier, _ = os.path.splitext(base_filename)
        run_output_dir = os.path.join(BASE_OUTPUT_DIR, run_identifier)
        heatmaps_dir = os.path.join(run_output_dir, "Heat Maps")
        dynamics_dir = os.path.join(run_output_dir, "Replicator Dynamics")
        equilibria_dir = os.path.join(run_output_dir, "Nash Equilibria")
        os.makedirs(heatmaps_dir, exist_ok=True)
        os.makedirs(dynamics_dir, exist_ok=True)
        os.makedirs(equilibria_dir, exist_ok=True)
        print(f"Output will be saved in: {run_output_dir}")
        return run_output_dir, heatmaps_dir, dynamics_dir, equilibria_dir
    except Exception as e:
        print(f"Error creating output directory: {e}")
        sys.exit(1)

# --- Main Analysis Function ---
def analyze_tournament_results(csv_file):
    """
    Analyze tournament results: Overall performance (incl. draws),
    heatmaps, replicator dynamics, Nash equilibria. Saves output.
    """
    run_output_dir, heatmaps_dir, dynamics_dir, equilibria_dir = create_output_directory(csv_file)

    print(f"Reading data from {csv_file}...")
    try:
        df = pd.read_csv(csv_file)
        # Ensure columns that should be numeric are numeric
        cols_to_numeric = ['Wins', 'Losses', 'Draws', 'Win %', 'Games Played']
        for col in cols_to_numeric:
            df[col] = pd.to_numeric(df[col], errors='coerce')
        # Drop rows where essential numeric data couldn't be parsed
        df.dropna(subset=cols_to_numeric, inplace=True)

    except FileNotFoundError:
        print(f"Error: CSV file not found at '{csv_file}'"); sys.exit(1)
    except Exception as e:
        print(f"Error reading or processing CSV file: {e}"); sys.exit(1)

    # --- Basic Data Extraction ---
    strategies = sorted(df['Strategy'].unique())
    openings = df['Opening'].unique()
    print(f"\nFound {len(strategies)} strategies: {', '.join(strategies)}")
    print(f"Found {len(openings)} openings: {', '.join(openings)}")

    # --- Overall Strategy Performance (Score / Total Games) ---
    print("\n--- Overall Strategy Performance (Score / Total Games | Win=1, Draw=0.5, Loss=0) ---")
    strategy_performance_score = {}
    for strategy in strategies:
        # Get all records where this strategy participated as 'Strategy'
        # The CSV is symmetric, so this covers all games played by the strategy.
        strategy_records_df = df[df['Strategy'] == strategy]

        if strategy_records_df.empty:
            print(f"Warning: No records found for strategy '{strategy}' as 'Strategy'. Skipping performance calculation.")
            strategy_performance_score[strategy] = 0.0 # Or handle as appropriate
            continue

        # Sum relevant columns from these records
        total_wins = strategy_records_df['Wins'].sum()
        total_draws = strategy_records_df['Draws'].sum()
        total_games_played = strategy_records_df['Games Played'].sum()

        # Calculate score: Win=1 point, Draw=0.5 points
        total_score = total_wins + 0.5 * total_draws

        # Calculate performance percentage
        performance_percentage = (total_score / total_games_played) * 100 if total_games_played > 0 else 0
        strategy_performance_score[strategy] = performance_percentage

    print("\nOverall Performance Score Percentage:")
    # Sort by the calculated score percentage
    for strategy, performance in sorted(strategy_performance_score.items(), key=lambda item: item[1], reverse=True):
        print(f"- {strategy}: {performance:.2f}%")

    # --- Strategy vs Opening Heatmap (Based on Win Rate: Wins / (Wins + Losses)) ---
    # This heatmap remains based on the win rate in decisive games for comparability
    print("\n--- Generating Strategy vs Opening Heatmap (Based on Win Rate vs All Opponents) ---")
    strategy_opening_matrix = pd.DataFrame(index=strategies, columns=openings, dtype=float)
    for strategy in strategies:
        for opening in openings:
            strat_wins_df = df[(df['Strategy'] == strategy) & (df['Opening'] == opening)]
            strat_losses_df = df[(df['Opponent'] == strategy) & (df['Opening'] == opening)]
            wins = strat_wins_df['Wins'].sum()
            losses = strat_losses_df['Wins'].sum() # Opponent's wins are strategy's losses
            decisive_games = wins + losses
            # Use Win Rate = Wins / (Wins + Losses) for this specific heatmap
            win_percentage = (wins / decisive_games) * 100 if decisive_games > 0 else 0
            strategy_opening_matrix.loc[strategy, opening] = win_percentage

    plt.figure(figsize=(max(10, len(openings)*1.5), max(8, len(strategies)*0.6)))
    sns.heatmap(strategy_opening_matrix, annot=True, fmt=".1f", cmap="viridis", linewidths=.5)
    plt.xlabel('Openings'); plt.ylabel('Strategies')
    plt.title('Win Rate % of Strategy within each Opening (Wins / (Wins+Losses))')
    plt.xticks(rotation=45, ha='right'); plt.yticks(rotation=0)
    plt.tight_layout()
    save_path = os.path.join(heatmaps_dir, '0_Strategy_Opening_WinRate_Heatmap.png')
    plt.savefig(save_path); plt.close()
    print(f"Saved: {save_path}")


    # --- Per Opening Analysis (Using Head-to-Head Win Rate from CSV for Payoffs) ---
    print("\n--- Per Opening Analysis (Using Head-to-Head Win Rate for Payoffs) ---")
    all_nash_equilibria = {}
    HAS_NASHPY = False
    try:
        import nashpy as nash
        print("Nashpy found. Performing Replicator Dynamics and Nash Equilibria analysis.")
        HAS_NASHPY = True
    except ImportError:
        print("Warning: nashpy module not found. Skipping Nashpy analysis.")

    for k, opening in enumerate(openings):
        print(f"\n--- Analyzing Opening: {opening} ---")
        opening_df = df[df['Opening'] == opening].copy()

        # --- 1. Matchup Heatmap (Head-to-Head Win %) ---
        print(f"Generating Matchup Heatmap for {opening}...")
        matchup_matrix_pct = pd.DataFrame(index=strategies, columns=strategies, dtype=float)
        for i, s1 in enumerate(strategies):
            for j, s2 in enumerate(strategies):
                if i == j: matchup_matrix_pct.loc[s1, s2] = 50.0
                else:
                    matchup = opening_df[(opening_df['Strategy'] == s1) & (opening_df['Opponent'] == s2)]
                    if not matchup.empty:
                         # Use the pre-calculated Win % (Wins / (Wins + Losses)) from CSV
                        matchup_matrix_pct.loc[s1, s2] = matchup['Win %'].iloc[0]
                    else:
                        matchup_matrix_pct.loc[s1, s2] = np.nan
                        print(f"Warning: Missing matchup data for {s1} vs {s2} in {opening}")

        plt.figure(figsize=(max(10, len(strategies)*0.8), max(8, len(strategies)*0.7)))
        sns.heatmap(matchup_matrix_pct, annot=True, fmt=".1f", cmap="viridis_r",
                   linewidths=.5, linecolor='lightgray', cbar_kws={'label': f'Win % for Row Player'})
        plt.xlabel('Opponent Strategy'); plt.ylabel('Strategy'); plt.title(f'Head-to-Head Win % ({opening})')
        plt.xticks(rotation=45, ha='right'); plt.yticks(rotation=0); plt.tight_layout()
        heatmap_filename = f'{k+1}_{opening.replace(" ", "_")}_Matchup_Heatmap.png'
        save_path = os.path.join(heatmaps_dir, heatmap_filename); plt.savefig(save_path); plt.close()
        print(f"Saved: {save_path}")

        # --- 2. Replicator Dynamics & Nash Equilibria ---
        if HAS_NASHPY:
            print(f"Calculating Replicator Dynamics and Nash Equilibria for {opening}...")
             # Payoff matrix A based on head-to-head win rate (row player's perspective)
            payoff_matrix_A = matchup_matrix_pct.to_numpy() / 100.0

            if np.isnan(payoff_matrix_A).any():
                print(f"Skipping Nashpy analysis for {opening} due to missing data (NaN).")
                all_nash_equilibria[opening] = {"error": "Skipped due to NaN in payoff matrix"}
                continue

            try:
                game = nash.Game(payoff_matrix_A) # Zero sum game

                # --- 2a. Replicator Dynamics ---
                # ... (Replicator dynamics plotting code remains the same) ...
                initial_pop = np.array([1/len(strategies)] * len(strategies))
                timepoints = np.linspace(0, 50, 20000)
                populations = game.replicator_dynamics(y0=initial_pop, timepoints=timepoints)
                pop_df = pd.DataFrame(populations, columns=strategies); pop_df['Generation'] = range(len(populations))
                pop_long = pop_df.melt(id_vars=['Generation'], value_vars=strategies, var_name='Strategy', value_name='Population Share')
                plt.figure(figsize=(12, 8)); sns.set_style("whitegrid")
                palette = sns.color_palette("husl", len(strategies))
                ax = sns.lineplot(data=pop_long, x='Generation', y='Population Share', hue='Strategy', linewidth=2.5, palette=palette)
                plt.title(f'Replicator Dynamics ({opening})', fontsize=16)
                plt.xlabel('Generation', fontsize=14); plt.ylabel('Population Share', fontsize=14); plt.ylim(0, 1.05)
                plt.legend(title='Strategy', bbox_to_anchor=(1.05, 1), loc='upper left', borderaxespad=0.)
                plt.grid(True, linestyle='--', alpha=0.7); plt.tight_layout(rect=[0, 0, 0.85, 1])
                rd_filename = f"{k+1}_{opening.replace(' ', '_')}_RD.png"
                save_path = os.path.join(dynamics_dir, rd_filename); plt.savefig(save_path, dpi=300); plt.close()
                print(f"Saved: {save_path}")


                # --- 2b. Nash Equilibria (Vertex Enumeration) ---
                print(f"Finding Nash Equilibria for {opening} using Vertex Enumeration...")
                opening_nes_list = []
                try:
                    equilibria_generator = game.vertex_enumeration()
                    print(f"Nash Equilibria found for {opening}:")
                    ne_count = 0
                    for i, eq in enumerate(equilibria_generator):
                        ne_count += 1
                        print(f"  NE {i+1}:")
                        row_ne_probs, col_ne_probs = eq[0], eq[1]
                        row_ne_dict = {strategies[idx]: prob for idx, prob in enumerate(row_ne_probs) if prob > 1e-4}
                        col_ne_dict = {strategies[idx]: prob for idx, prob in enumerate(col_ne_probs) if prob > 1e-4}
                        print("    Row Player Strategy:", {s: f"{p:.3f}" for s, p in row_ne_dict.items()})
                        opening_nes_list.append({
                            "equilibrium_index": i + 1,
                            "row_strategy": row_ne_dict,
                            "column_strategy": col_ne_dict # Keep col strategy for completeness
                        })
                    if ne_count == 0: print("  No Nash Equilibria found by vertex enumeration.")
                    all_nash_equilibria[opening] = opening_nes_list

                except OverflowError:
                    print(f"Error (Overflow) during vertex enumeration for {opening}. Matrix may be too large/complex.")
                    all_nash_equilibria[opening] = {"error": "OverflowError during vertex enumeration"}
                except Exception as ve_error:
                    print(f"Error during vertex enumeration for {opening}: {ve_error}")
                    all_nash_equilibria[opening] = {"error": str(ve_error)}

            except ValueError as nan_error:
                 print(f"Skipping Nashpy analysis for {opening} due to error (likely NaN): {nan_error}")
                 all_nash_equilibria[opening] = {"error": "Skipped due to NaN in payoff matrix"}
            except Exception as e:
                print(f"Error during Nashpy analysis for {opening}: {e}")
                all_nash_equilibria[opening] = {"error": str(e)}

    # --- Save Nash Equilibria to JSON ---
    if HAS_NASHPY and all_nash_equilibria:
        ne_filename = "nash_equilibria.json"
        ne_save_path = os.path.join(equilibria_dir, ne_filename)
        print(f"\n--- Saving Nash Equilibria to JSON ---")
        try:
            with open(ne_save_path, 'w') as f:
                # Convert numpy types to native Python types for JSON serialization
                 serializable_nes = {}
                 for opening, nes in all_nash_equilibria.items():
                     if isinstance(nes, list):
                         serializable_nes[opening] = [
                            {
                                "equilibrium_index": ne["equilibrium_index"],
                                "row_strategy": {k: float(v) for k, v in ne["row_strategy"].items()},
                                "column_strategy": {k: float(v) for k, v in ne["column_strategy"].items()}
                            } for ne in nes
                         ]
                     else: # Handle error dictionaries
                         serializable_nes[opening] = nes

                 json.dump(serializable_nes, f, indent=4)
            print(f"Saved: {ne_save_path}")
        except Exception as e:
            print(f"Error saving Nash Equilibria to JSON: {e}")
    elif not HAS_NASHPY:
         print("\nSkipped saving Nash Equilibria (Nashpy not installed).")
    else:
         print("\nNo Nash Equilibria results to save.")

    print("\n--- Analysis Complete ---")
    return df

# --- Main execution block ---
if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Analyze Quoridor tournament results.")
    parser.add_argument("csv_file", help="Path to the tournament results CSV file.")
    args = parser.parse_args()

    if not os.path.exists(args.csv_file): print(f"Error: Input CSV file not found at '{args.csv_file}'"); sys.exit(1)
    if not os.path.isfile(args.csv_file): print(f"Error: Provided path '{args.csv_file}' is not a file."); sys.exit(1)

    analyze_tournament_results(args.csv_file)