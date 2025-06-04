from typing import TypeAlias, Union, Any
from velopix import TrackFollowing, GraphDFS, SearchByTripletTrie

MetricsDict: TypeAlias = dict[str, int|float|bool]
pMapType: TypeAlias = dict[str, tuple[type[int]|type[float]|type[bool],Any]]
pMap: TypeAlias = dict[str, int | float | bool]
EventType: TypeAlias = list[dict[str, Any]]
ReconstructionAlgorithmsType: TypeAlias = Union[TrackFollowing, GraphDFS, SearchByTripletTrie]
ValidationResults: TypeAlias = dict[str, dict[str, list[dict[str, Union[int, float, str]]]]]
ValidationResultsNested: TypeAlias = dict[str, dict[str, list[dict[str, Union[int, float, str]]]|dict[str, list[Union[int, float, str]]]]]
ConfigType: TypeAlias = dict[str, tuple[Union[type[float], type[bool], type[int]], Any]]
boundType: TypeAlias = dict[str, tuple[Union[int, float], Union[int, float]]|Any]