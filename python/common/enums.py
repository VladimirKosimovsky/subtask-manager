from enum import Enum

from pydantic import BaseModel


class TaskTypeDef(BaseModel):
    id: int
    extensions: list[str]


class TaskType(Enum):
    """
    SQL - SQL scripts, runnable on RDBMS, OLAP, NoSQL systems such as PostgreSQL, DuckDB, Vertica, ClickHouse etc.
    SHELL - Shell scripts, runnable on Unix-like systems such as Linux, macOS, etc.
    POWERSHELL - PowerShell scripts, runnable on Windows systems
    PYTHON - Python scripts, executable on the remote systems
    GRAPQHQL - GraphQL queries, runnable on any web servers
    JSON - JSON files, may contain the body of the POST requests, configurations, mappings etc.
    YAML - YAML files, may contain configurations, mappings, etc.
    """

    SQL = TaskTypeDef(id=0, extensions=["sql", "psql", "tsql", "plpgsql"])
    SHELL = TaskTypeDef(id=1, extensions=["sh"])
    POWERSHELL = TaskTypeDef(id=2, extensions=["ps1"])
    PYTHON = TaskTypeDef(id=3, extensions=["py"])
    GRAPHQL = TaskTypeDef(id=4, extensions=["gql", "graphql"])
    JSON = TaskTypeDef(id=5, extensions=["json", "jsonl"])
    YAML = TaskTypeDef(id=6, extensions=["yaml", "yml"])

    @property
    def id(self) -> int:
        return self.value.id

    @property
    def extensions(self) -> list[str]:
        return self.value.extensions

    @classmethod
    def from_extension(cls, ext: str) -> "TaskType":
        """Lookup by file extension (case-insensitive)."""
        ext = ext.lower()
        for member in cls:
            if ext in member.extensions:
                return member
        raise ValueError(f"Unknown task type for extension: {ext}")


class EtlStageDef(BaseModel):
    id: int
    folder_names: list[str]


class EtlStage(Enum):
    """
    SETUP - preliminary setup of temp folders, databases, objects, etc.
    EXTRACT - extract data from source systems
    TRANSFORM - transform data into a format suitable for loading
    LOAD - load data into target systems
    CLEANUP - cleanup temporary files and resources
    POST_PROCESSING - post-processing tasks on the target system's side
    """

    SETUP = EtlStageDef(id=0, folder_names=["00_setup", "setup", "s", "00"])
    EXTRACT = EtlStageDef(id=1, folder_names=["01_extract", "extract", "e", "01"])
    TRANSFORM = EtlStageDef(id=2, folder_names=["02_transform", "transform", "t", "02"])
    LOAD = EtlStageDef(id=3, folder_names=["03_load", "load", "l", "03"])
    CLEANUP = EtlStageDef(id=4, folder_names=["04_cleanup", "cleanup", "c", "04"])
    POST_PROCESSING = EtlStageDef(
        id=5, folder_names=["05_post_processing", "post_processing", "pp", "05"]
    )

    @property
    def id(self) -> int:
        return self.value.id

    @property
    def folder_names(self) -> list[str]:
        return self.value.folder_names

    @classmethod
    def from_folder_name(cls, alias: str) -> "EtlStage":
        """Lookup by folder name alias (case-insensitive)."""
        alias = alias.lower()
        for member in cls:
            if alias in (name.lower() for name in member.folder_names):
                return member
        raise ValueError(f"Unknown ETL stage alias: {alias}")


class SystemTypeDef(BaseModel):
    id: int
    aliases: list[str]


class SystemType(Enum):
    """ """

    PG = SystemTypeDef(
        id=0, aliases=["pg", "postgres", "pg_dwh", "postgres_db", "postgresdb"]
    )
    DUCK = SystemTypeDef(id=1, aliases=["duck", "duckdb"])
    VERTICA = SystemTypeDef(id=2, aliases=["vertica"])

    @property
    def id(self) -> int:
        return self.value.id

    @property
    def aliases(self) -> list[str]:
        return self.value.aliases

    @classmethod
    def from_alias(cls, alias: str) -> "SystemType":
        """Lookup by folder name alias (case-insensitive)."""
        alias = alias.lower()
        for member in cls:
            if alias in (name.lower() for name in member.aliases):
                return member
        raise ValueError(f"Unknown system type alias: {alias}")
