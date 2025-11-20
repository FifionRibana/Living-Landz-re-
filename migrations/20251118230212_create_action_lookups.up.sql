-- Add up migration script here
CREATE TABLE actions.action_statuses (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO actions.action_statuses (id, name) VALUES
    (1, 'InProgress'),
    (2, 'Pending'),
    (3, 'Completed'),
    (4, 'Failed');

CREATE TABLE actions.action_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO actions.action_types (id, name) VALUES
    (1, 'BuildBuilding'),
    (2, 'BuildRoad'),
    (3, 'MoveUnit'),
    (4, 'SendMessage'),
    (5, 'HarvestResource'),
    (6, 'CraftResource');

CREATE TABLE actions.action_specific_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO actions.action_specific_types (id, name) VALUES
    (1, 'BuildBuilding'),
    (2, 'BuildRoad'),
    (3, 'MoveUnit'),
    (4, 'SendMessage'),
    (5, 'HarvestResource'),
    (6, 'CraftResource');