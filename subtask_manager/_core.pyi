from pathlib import Path

from typing_extensions import override

class TaskType:
    id: int
    name: str
    extensions: list[str]
    Sql:TaskType
    Shell:TaskType
    Powershell:TaskType
    Python:TaskType
    Graphql:TaskType
    Json:TaskType
    Yaml:TaskType
    Other:TaskType
    # Constructor - private to prevent direct instantiation
    def __init__(self, *args: object, **kwargs: object) -> None: ...
    
    @classmethod
    def from_extension(
        cls, extension: str,
    ) -> SystemType: ...
    
    # String representation
    @override
    def __str__(self) -> str: ...
    @override
    def __repr__(self) -> str: ...

    # Comparisons
    @override
    def __eq__(self, other: object) -> bool: ...
    @override
    def __ne__(self, other: object) -> bool: ...
    @override
    def __hash__(self) -> int: ...
class SystemType:
    id: int
    name: str
    aliases: list[str]
    Clickhouse: SystemType
    Duckdb: SystemType
    MySQL: SystemType
    OracleDB: SystemType
    PostgreSQL: SystemType
    SQLite: SystemType
    SqlServer: SystemType
    Vertica: SystemType
    Other: SystemType
    # Constructor - private to prevent direct instantiation
    def __init__(self, *args: object, **kwargs: object) -> None: ...
    
    @classmethod
    def from_alias(
        cls, alias: str,
    ) -> SystemType: ...
    
    # String representation
    @override
    def __str__(self) -> str: ...
    @override
    def __repr__(self) -> str: ...

    # Comparisons
    @override
    def __eq__(self, other: object) -> bool: ...
    @override
    def __ne__(self, other: object) -> bool: ...
    @override
    def __hash__(self) -> int: ...
    

class EtlStage:
    id: int
    name: str
    aliases: list[str]
    # Enum variants as class attributes
    Extract: EtlStage
    Transform: EtlStage
    Load: EtlStage
    Setup: EtlStage
    Cleanup: EtlStage
    Postprocessing: EtlStage

    # Constructor - private to prevent direct instantiation
    def __init__(self, *args: object, **kwargs: object) -> None: ...

    # String representation
    @override
    def __str__(self) -> str: ...
    @override
    def __repr__(self) -> str: ...

    # Comparisons
    @override
    def __eq__(self, other: object) -> bool: ...
    @override
    def __ne__(self, other: object) -> bool: ...
    @override
    def __hash__(self) -> int: ...
    
    @classmethod
    def from_alias(
        cls, alias: str,
    ) -> EtlStage: ...

class Subtask:
    stage: EtlStage | None
    entity: str | None
    system_type: SystemType | None
    task_type: TaskType | None
    is_common: bool
    name: str
    path: str
    command: str | None

    def __init__(
        self,
        stage: EtlStage | None = None,
        entity: str | None = None,
        system_type: SystemType | None = None,
        task_type: TaskType | None = None,
        is_common: bool = False,
        name: str = "",
        path: str = "",
        command: str | None = None,
    ) -> None: ...
    @override
    def __repr__(self) -> str: ...
    @override
    def __str__(self) -> str: ...
    
    def apply_parameters(
        self, 
        params:dict, 
        styles:list[str] | None = None, 
        ignore_missing:bool = False
    ) -> None:
        ...
    
    def get_params(
        self, 
        styles:list[str] | None = None
    ) -> set[str]:
        ...

class SubtaskManager:
    base_path: str
    subtasks: list[Subtask]
    classifier: FileClassifier
    
    def __init__(self, base_path: str | Path) -> None: ...

    def get_tasks(
        self,
        etl_stage: EtlStage | None = None,
        entity: str | None = None,
        system_type: SystemType | None = None,
        task_type: TaskType | None = None,
        is_common: bool | None = None,
        include_common: bool | None = True,
    ) -> list[Subtask]: ...
    def get_task(
        self,
        name: str,
        entity: str | None = None,
    ) -> Subtask: ...
    

class FileScanner:
    """Scanner for finding files with specific extensions."""
    
    def __init__(self, extensions: list[str]) -> None:
        """
        Initialize FileScanner with file extensions to search for.
        
        Args:
            extensions: List of file extensions (with or without leading dot)
        """
        ...
    
    def scan_files(self, base_dir: str | Path) -> list[str]:
        """
        Scan directory recursively for files with matching extensions.
        
        Args:
            base_dir: Root directory to scan (string path or pathlib.Path)
            
        Returns:
            List of absolute file paths
        """
        ...
    
    @property
    def extensions(self) -> list[str]:
        """Get the normalized extensions this scanner searches for."""
        ...

class FileClassifier:
    """Classifier for converting file paths into Subtask objects based on folder structure."""
    
    base_path: str
    
    def __init__(self, base_path: str | Path) -> None:
        """
        Initialize FileClassifier with a base path.
        
        Args:
            base_path: Base directory path (string path or pathlib.Path)
        """
        ...
    
    def classify(self, file_path: str | Path) -> Subtask:
        """
        Classify a file path into a Subtask based on its location relative to base_path.
        
        Args:
            file_path: Path to the file to classify (string path or pathlib.Path)
            
        Returns:
            A Subtask object with extracted metadata
            
        Raises:
            RuntimeError: If the folder structure is invalid or task type cannot be determined
        """
        ...
    
    @override
    def __repr__(self) -> str: ...