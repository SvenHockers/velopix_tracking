# import os
# import pandas as pd
# from typing import Any, cast, Callable
# from .custom_types import *

# def save_to_file(results: list[ValidationResults], directory: str, output_func: str = "output_aggregates", overwrite: bool = False) -> None:
#     """
#     Wrapped function logic to call, generate dataframes and save them to an output directory automatticly
#     """
#     func: Callable[[[ValidationResults]], Any] = getattr(output_func, None) # type: ignore
#     if not callable(func): # type: ignore
#         raise ValueError(f"Output function '{output_func}' not found.")
    
#     if output_func == "output_aggregates":
#         df1, df2 = func(results)
#         files = [os.path.join(directory, "overall.csv"),
#                     os.path.join(directory, "category.csv")]
#         dataframes: list[pd.DataFrame] = [df1, df2]
#     elif output_func == "output_distrubutions":
#         df1, df2, df3 = func(results)
#         files = [os.path.join(directory, "overall.csv"),
#                     os.path.join(directory, "category.csv"),
#                     os.path.join(directory, "event_distribution.csv")]
#         dataframes: list[pd.DataFrame] = [df1, df2, df3]
#     else:
#         raise NotImplementedError(f"output_func: {output_func} does not exist")
    
#     if not os.path.exists(directory):
#         os.makedirs(directory)
    
#     for file in files:
#         if not overwrite and os.path.exists(file):
#             raise FileExistsError(f"The file '{file}' already exists. Set 'overwrite=True' to overwrite it.")
    
#     for df, file in zip(dataframes, files):
#         df.to_csv(file, index=False)

# def output_aggregates(results: list[ValidationResultsNested]) -> tuple[pd.DataFrame, pd.DataFrame]:
#     """
#     Given a list of JSON result dictionaries from multiple runs,
#     return two Dataframes:
#     - overall_df: one row per run with overall metrics and parameters.
#     - category_df: one row per category per run, also including parameters.
#     """
#     overall_rows: list[dict[str, int|float]] = []
#     category_rows: list[dict[str, Any]] = []
    
#     for run_id, res in enumerate(results):
#         # extract solver parameters if available.
#         params: dict[str, int|float] = cast(dict[str, int|float], res.get('parameters', {}))
#         overall_row: dict[str, int|float] = {
#             'run_id': run_id,
#             'inference_time': cast(float, res.get('inference_time')),
#             'total_tracks': cast(int, res.get('total_tracks')),
#             'total_ghosts': cast(int, res.get('total_ghosts')),
#             'overall_ghost_rate': cast(float, res.get('overall_ghost_rate')),
#             'event_avg_ghost_rate': cast(float, res.get('event_avg_ghost_rate'))
#         }
#         # merge parameters into the overall summary.
#         overall_row.update(params)
#         overall_rows.append(overall_row)
        
#         # process each category: add run_id and merge parameters.
#         for cat in res.get('categories', []):
#             cat_row = cat.copy() #
#             cat_row['run_id'] = run_id
#             cat_row.update(params)
#             category_rows.append(cat_row)
    
#     overall_df = pd.DataFrame(overall_rows)
#     category_df = pd.DataFrame(category_rows)
    
#     return overall_df, category_df

# def output_distributions(results: ValidationResults) -> tuple[pd.DataFrame, pd.DataFrame, pd.DataFrame]:
#     """
#     Given a list of JSON result dictionaries from multiple runs using the nested validaton method,
#     return three Dataframes:
#     - overall_df: one row per run with overall metrics and parameters.
#     - category_df: one row per category per run, also including parameters.
#     - distrubition_df: one row per run storing distrubution data. 
#     """
#     overall_rows = []
#     category_rows = []
#     distributions_rows = []
#     for run_id, res in enumerate(results):
#         # extract solver parameters if available.
#         params = res.get('parameters', {})
#         overall_row = {
#             'run_id': run_id,
#             'inference_time': res.get('inference_time'),
#             'total_tracks': res.get('total_tracks'),
#             'total_ghosts': res.get('total_ghosts'),
#             'overall_ghost_rate': res.get('overall_ghost_rate'),
#             'event_avg_ghost_rate': res.get('event_avg_ghost_rate')
#         }
#         # merge parameters into the overall summary.
#         overall_row.update(params)
#         overall_rows.append(overall_row)
        
#         # process each category: add run_id and merge parameters.
#         for cat in res.get('categories', []):
#             cat_row = cat.copy()
#             cat_row['run_id'] = run_id
#             cat_row.update(params)
#             category_rows.append(cat_row)

#         # here we capture individual event results, calculate statistical data regarding it and store it to a row -> this is done to reduce the data overhead
#         events = res.get('events', [])
#         rows = []
#         for event_id, event_list in events.items():
#             for entry in event_list:
#                 entry_copy = entry.copy()
#                 entry_copy["event_id"] = int(event_id)
#                 rows.append(entry_copy)
#         df_events = pd.DataFrame(rows)
#         df_events["label"] = df_events['label'].astype(str) # for some weird reason this is an object so have to cast it to str instead 
#         dist_summary = {}

#         if not df_events.empty:
#             if 'label' in df_events.columns:
#                 for label, group in df_events.groupby('label'):
#                     numeric_cols = group.select_dtypes(include=['number']).columns #ensure we only use numeric columns
#                     for col in numeric_cols:
#                         dist_summary[f"{label}_{col}_mean"] = group[col].mean()
#                         dist_summary[f"{label}_{col}_std"] = group[col].std()
#                         dist_summary[f"{label}_{col}_min"] = group[col].min()
#                         dist_summary[f"{label}_{col}_max"] = group[col].max()
#                         dist_summary[f"{label}_{col}_median"] = group[col].median()
#                         q25 = group[col].quantile(0.25)
#                         q75 = group[col].quantile(0.75)
#                         dist_summary[f"{label}_{col}_q25"] = q25
#                         dist_summary[f"{label}_{col}_q75"] = q75
#                         dist_summary[f"{label}_{col}_iqr"] = q75 - q25
#                         dist_summary[f"{label}_{col}_skew"] = group[col].skew()
#                         dist_summary[f"{label}_{col}_kurtosis"] = group[col].kurtosis()
#             else:
#                 # Fallback: if there is no label column, compute overall statistics per numeric column. May consider using a different fallback
#                 print("Fallback triggered!")
#                 numeric_cols = df_events.select_dtypes(include=['number']).columns
#                 for col in numeric_cols:
#                     dist_summary[f"{col}_mean"] = df_events[col].mean()
#                     dist_summary[f"{col}_std"] = df_events[col].std()
#                     dist_summary[f"{col}_min"] = df_events[col].min()
#                     dist_summary[f"{col}_max"] = df_events[col].max()
#                     dist_summary[f"{col}_median"] = df_events[col].median()
#                     q25 = df_events[col].quantile(0.25)
#                     q75 = df_events[col].quantile(0.75)
#                     dist_summary[f"{col}_q25"] = q25
#                     dist_summary[f"{col}_q75"] = q75
#                     dist_summary[f"{col}_iqr"] = q75 - q25
#                     dist_summary[f"{col}_skew"] = df_events[col].skew()
#                     dist_summary[f"{col}_kurtosis"] = df_events[col].kurtosis()
#         dist_summary["number_events"] = df_events["event_id"].max()
#         dist_summary["run_id"] = run_id
#         distributions_rows.append(dist_summary)
        
#     overall_df = pd.DataFrame(overall_rows)
#     category_df = pd.DataFrame(category_rows)
#     distributions_df = pd.DataFrame(distributions_rows)

#     return overall_df, category_df, distributions_df
