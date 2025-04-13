import pandas as pd
from typing import Union
from .custom_types import *
"""
Here is a list of available metrics:
    'avg_hiteff'
    'avg_purity'
    'avg_recoeff'
    'clone_percentage'
    'hit_eff_percentage'
    'label': 'long_strange',
    'n_clones'
    'n_particles'
    'n_reco'                         
    'purityT'         
    'recoeffT'                        
"""
class EventMetricsCalculator:
    def __init__(self, validation_results: ValidationResults):
        self.validation_results = validation_results
        self.df_events = self._create_events_dataframe()

    def _create_events_dataframe(self):
        events: dict[str, list[dict[str, Union[int, float, str]]]] = self.validation_results.get("events", {})
        events_list = [entry for event_list in events.values() for entry in event_list]
        return pd.DataFrame(events_list)

    def compute_aggregations(self) -> pd.DataFrame|None:
        if self.df_events.empty or 'label' not in self.df_events.columns:
            raise(AssertionError("Something went wrong (Sorry was to lazy to define a helpfull error)"))

        numeric_cols = self.df_events.select_dtypes(include=['number']).columns

        def q25(x: int|float) -> float:
            return x.quantile(0.25) # type: ignore
        def q75(x: int|float) -> float:
            return x.quantile(0.75) # type: ignore
        def my_skew(x: int|float) -> float:
            return x.skew() # type: ignore
        def my_kurtosis(x: int|float) -> float:
            return x.kurtosis() # type: ignore
        
        q25.__name__ = 'q25'
        q75.__name__ = 'q75'
        my_skew.__name__ = 'skew'
        my_kurtosis.__name__ = 'kurtosis'

        aggregations: list[str|function] = [
            'mean',
            'std',
            'min',
            'max',
            'median',
            q25,
            q75,
            my_skew,
            my_kurtosis
        ]
        agg_df: pd.DataFrame = self.df_events.groupby("label")[numeric_cols].agg(aggregations) # type: ignore

        # Compute IQR and add as an extra column
        iqr_df: pd.DataFrame = agg_df.xs('q75', level=1, axis=1) - agg_df.xs('q25', level=1, axis=1) # type: ignore
        iqr_df.columns: pd.DataFrame = pd.MultiIndex.from_product([iqr_df.columns, ['iqr']]) # type: ignore
        return pd.concat([agg_df, iqr_df], axis=1) # type: ignore

    def flatten_aggregations(self, agg_df: pd.DataFrame) -> MetricsDict:
        metrics: MetricsDict = {}
        for label, row in agg_df.iterrows(): # type: ignore
            for col, stat in agg_df.columns:
                metrics[f"{label}_{col}_{stat}"] = row[(col, stat)] # type: ignore
        return metrics

    def compute_average_metric(self, metrics: MetricsDict, col: str, stat: str):
        matching_values = [v for k, v in metrics.items() if k.endswith(f"_{col}_{stat}")]
        if matching_values:
            return sum(matching_values) / len(matching_values)
        raise(AssertionError("Something went wrong (Sorry was to lazy to define a helpfull error)"))

    def get_metric(self, metric: str ="clone_percentage", stat: str ="std"):
        # Note this metric returns the avg of this metric (ie: sum(metric) / lwn(metric))
        agg_df = self.compute_aggregations()
        if agg_df is None:
            raise(AssertionError("Something went wrong (Sorry was to lazy to define a helpfull error)"))
        metrics = self.flatten_aggregations(agg_df)
        return self.compute_average_metric(metrics, metric, stat)
