--
-- PostgreSQL database dump
--

\restrict WdS0PDWDAvqS22er524zGwjOV6Dq7pMCs5lVzTreYPxP0bB05GLXDJktRrkinrc

-- Dumped from database version 17.6
-- Dumped by pg_dump version 17.6

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: actions; Type: SCHEMA; Schema: -; Owner: living_landz_srv
--

CREATE SCHEMA actions;


ALTER SCHEMA actions OWNER TO living_landz_srv;

--
-- Name: SCHEMA actions; Type: COMMENT; Schema: -; Owner: living_landz_srv
--

COMMENT ON SCHEMA actions IS 'Syst├¿me des actions planifi├®es';


--
-- Name: buildings; Type: SCHEMA; Schema: -; Owner: living_landz_srv
--

CREATE SCHEMA buildings;


ALTER SCHEMA buildings OWNER TO living_landz_srv;

--
-- Name: SCHEMA buildings; Type: COMMENT; Schema: -; Owner: living_landz_srv
--

COMMENT ON SCHEMA buildings IS 'B├ótiments et constructions';


--
-- Name: game; Type: SCHEMA; Schema: -; Owner: living_landz_srv
--

CREATE SCHEMA game;


ALTER SCHEMA game OWNER TO living_landz_srv;

--
-- Name: SCHEMA game; Type: COMMENT; Schema: -; Owner: living_landz_srv
--

COMMENT ON SCHEMA game IS 'Configuration et constantes du jeu';


--
-- Name: organizations; Type: SCHEMA; Schema: -; Owner: living_landz_srv
--

CREATE SCHEMA organizations;


ALTER SCHEMA organizations OWNER TO living_landz_srv;

--
-- Name: SCHEMA organizations; Type: COMMENT; Schema: -; Owner: living_landz_srv
--

COMMENT ON SCHEMA organizations IS 'Syst├¿me des organisations (territoires, guildes, ordres religieux, etc.)';


--
-- Name: resources; Type: SCHEMA; Schema: -; Owner: living_landz_srv
--

CREATE SCHEMA resources;


ALTER SCHEMA resources OWNER TO living_landz_srv;

--
-- Name: SCHEMA resources; Type: COMMENT; Schema: -; Owner: living_landz_srv
--

COMMENT ON SCHEMA resources IS 'Ressources et ├®conomie';


--
-- Name: terrain; Type: SCHEMA; Schema: -; Owner: living_landz_srv
--

CREATE SCHEMA terrain;


ALTER SCHEMA terrain OWNER TO living_landz_srv;

--
-- Name: SCHEMA terrain; Type: COMMENT; Schema: -; Owner: living_landz_srv
--

COMMENT ON SCHEMA terrain IS 'Terrain, chunks, cells';


--
-- Name: units; Type: SCHEMA; Schema: -; Owner: living_landz_srv
--

CREATE SCHEMA units;


ALTER SCHEMA units OWNER TO living_landz_srv;

--
-- Name: SCHEMA units; Type: COMMENT; Schema: -; Owner: living_landz_srv
--

COMMENT ON SCHEMA units IS 'Syst├¿me des unit├®s et personnages';


--
-- Name: cleanup_leader_on_officer_removal(); Type: FUNCTION; Schema: organizations; Owner: living_landz_srv
--

CREATE FUNCTION organizations.cleanup_leader_on_officer_removal() RETURNS trigger
    LANGUAGE plpgsql
    AS $$

DECLARE

    org_type_id SMALLINT;

    was_leader_role BOOLEAN;

BEGIN

    -- Get organization type

    SELECT organization_type_id INTO org_type_id

    FROM organizations.organizations

    WHERE id = OLD.organization_id;



    -- Check if this was a leader role

    SELECT is_leader_role INTO was_leader_role

    FROM organizations.organization_role_compatibility

    WHERE organization_type_id = org_type_id

    AND role_type_id = OLD.role_type_id;



    -- If it was a leader role, clear the organization's leader_unit_id

    IF was_leader_role THEN

        UPDATE organizations.organizations

        SET leader_unit_id = NULL

        WHERE id = OLD.organization_id

        AND leader_unit_id = OLD.unit_id;

    END IF;



    RETURN OLD;

END;

$$;


ALTER FUNCTION organizations.cleanup_leader_on_officer_removal() OWNER TO living_landz_srv;

--
-- Name: remove_officer_on_member_removal(); Type: FUNCTION; Schema: organizations; Owner: living_landz_srv
--

CREATE FUNCTION organizations.remove_officer_on_member_removal() RETURNS trigger
    LANGUAGE plpgsql
    AS $$

BEGIN

    IF OLD.membership_status = 'active' AND NEW.membership_status != 'active' THEN

        DELETE FROM organizations.officers

        WHERE organization_id = NEW.organization_id

        AND unit_id = NEW.unit_id;

    END IF;

    RETURN NEW;

END;

$$;


ALTER FUNCTION organizations.remove_officer_on_member_removal() OWNER TO living_landz_srv;

--
-- Name: update_organization_population(); Type: FUNCTION; Schema: organizations; Owner: living_landz_srv
--

CREATE FUNCTION organizations.update_organization_population() RETURNS trigger
    LANGUAGE plpgsql
    AS $$

DECLARE

    org_id BIGINT;

BEGIN

    -- Get organization_id from either NEW or OLD record

    IF TG_OP = 'DELETE' THEN

        org_id := OLD.organization_id;

    ELSE

        org_id := NEW.organization_id;

    END IF;



    -- Update population count

    UPDATE organizations.organizations

    SET population = (

        SELECT COUNT(*)

        FROM organizations.members

        WHERE organization_id = org_id

        AND membership_status = 'active'

    )

    WHERE id = org_id;



    IF TG_OP = 'DELETE' THEN

        RETURN OLD;

    ELSE

        RETURN NEW;

    END IF;

END;

$$;


ALTER FUNCTION organizations.update_organization_population() OWNER TO living_landz_srv;

--
-- Name: update_organization_timestamp(); Type: FUNCTION; Schema: organizations; Owner: living_landz_srv
--

CREATE FUNCTION organizations.update_organization_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$

BEGIN

    NEW.updated_at = NOW();

    RETURN NEW;

END;

$$;


ALTER FUNCTION organizations.update_organization_timestamp() OWNER TO living_landz_srv;

--
-- Name: update_territory_area(); Type: FUNCTION; Schema: organizations; Owner: living_landz_srv
--

CREATE FUNCTION organizations.update_territory_area() RETURNS trigger
    LANGUAGE plpgsql
    AS $$

BEGIN

    UPDATE organizations.organizations

    SET total_area_km2 = (

        SELECT COUNT(*) * 0.01

        FROM organizations.territory_cells

        WHERE organization_id = NEW.organization_id

    )

    WHERE id = NEW.organization_id;

    RETURN NEW;

END;

$$;


ALTER FUNCTION organizations.update_territory_area() OWNER TO living_landz_srv;

--
-- Name: update_updated_at_column(); Type: FUNCTION; Schema: organizations; Owner: living_landz_srv
--

CREATE FUNCTION organizations.update_updated_at_column() RETURNS trigger
    LANGUAGE plpgsql
    AS $$

BEGIN

    NEW.updated_at = NOW();

    RETURN NEW;

END;

$$;


ALTER FUNCTION organizations.update_updated_at_column() OWNER TO living_landz_srv;

--
-- Name: validate_officer_role(); Type: FUNCTION; Schema: organizations; Owner: living_landz_srv
--

CREATE FUNCTION organizations.validate_officer_role() RETURNS trigger
    LANGUAGE plpgsql
    AS $$

DECLARE

    org_type_id SMALLINT;

    is_compatible BOOLEAN;

    is_leader_role BOOLEAN;

    current_leader_count INT;

    is_member BOOLEAN;

    is_leader BOOLEAN;

BEGIN

    -- Get organization type

    SELECT organization_type_id INTO org_type_id

    FROM organizations.organizations

    WHERE id = NEW.organization_id;



    -- Check if unit is a member or the leader

    SELECT EXISTS (

        SELECT 1 FROM organizations.members

        WHERE organization_id = NEW.organization_id

        AND unit_id = NEW.unit_id

        AND membership_status = 'active'

    ) INTO is_member;



    SELECT EXISTS (

        SELECT 1 FROM organizations.organizations

        WHERE id = NEW.organization_id

        AND leader_unit_id = NEW.unit_id

    ) INTO is_leader;



    IF NOT (is_member OR is_leader) THEN

        RAISE EXCEPTION 'Unit must be a member or leader of the organization before becoming an officer';

    END IF;



    -- Check if role is compatible with organization type

    SELECT

        COUNT(*) > 0,

        COALESCE(MAX(orc.is_leader_role), false)

    INTO is_compatible, is_leader_role

    FROM organizations.organization_role_compatibility orc

    WHERE orc.organization_type_id = org_type_id

    AND orc.role_type_id = NEW.role_type_id;



    IF NOT is_compatible THEN

        RAISE EXCEPTION 'Role type % is not compatible with this organization type', NEW.role_type_id;

    END IF;



    -- If this is a leader role, ensure there's only one leader

    IF is_leader_role THEN

        SELECT COUNT(*) INTO current_leader_count

        FROM organizations.officers o

        JOIN organizations.organization_role_compatibility orc

            ON orc.role_type_id = o.role_type_id

            AND orc.organization_type_id = org_type_id

        WHERE o.organization_id = NEW.organization_id

        AND orc.is_leader_role = true

        AND (TG_OP = 'INSERT' OR o.id != NEW.id);



        IF current_leader_count > 0 THEN

            RAISE EXCEPTION 'Organization already has a leader';

        END IF;



        -- Update the organization's leader_unit_id

        UPDATE organizations.organizations

        SET leader_unit_id = NEW.unit_id

        WHERE id = NEW.organization_id;

    END IF;



    RETURN NEW;

END;

$$;


ALTER FUNCTION organizations.validate_officer_role() OWNER TO living_landz_srv;

--
-- Name: validate_organization_constraints(); Type: FUNCTION; Schema: organizations; Owner: living_landz_srv
--

CREATE FUNCTION organizations.validate_organization_constraints() RETURNS trigger
    LANGUAGE plpgsql
    AS $$

DECLARE

    org_type RECORD;

    territory_count INT;

    current_population INT;

    current_area DECIMAL;

BEGIN

    -- Get organization type info

    SELECT * INTO org_type

    FROM organizations.organization_types

    WHERE id = NEW.organization_type_id;



    -- Check if requires territory

    IF org_type.requires_territory THEN

        SELECT COUNT(*) INTO territory_count

        FROM organizations.territory_cells

        WHERE organization_id = NEW.id;



        -- Non-territorial organizations must have headquarters in existing org

        IF territory_count = 0 AND NOT org_type.category = 'territorial' THEN

            IF NEW.headquarters_cell_q IS NULL THEN

                RAISE EXCEPTION 'Organization type % requires a headquarters location', org_type.name;

            END IF;

        END IF;

    END IF;



    -- Validate parent relationship

    IF NEW.parent_organization_id IS NOT NULL THEN

        IF NOT org_type.can_have_parent THEN

            RAISE EXCEPTION 'Organization type % cannot have a parent organization', org_type.name;

        END IF;

    END IF;



    -- Check minimum population (if updating)

    IF TG_OP = 'UPDATE' AND org_type.min_population IS NOT NULL THEN

        IF NEW.population < org_type.min_population THEN

            RAISE EXCEPTION 'Organization type % requires minimum population of %, current: %',

                org_type.name, org_type.min_population, NEW.population;

        END IF;

    END IF;



    -- Check minimum area (if updating)

    IF TG_OP = 'UPDATE' AND org_type.min_area_km2 IS NOT NULL THEN

        IF NEW.total_area_km2 < org_type.min_area_km2 THEN

            RAISE EXCEPTION 'Organization type % requires minimum area of % km┬▓, current: % km┬▓',

                org_type.name, org_type.min_area_km2, NEW.total_area_km2;

        END IF;

    END IF;



    RETURN NEW;

END;

$$;


ALTER FUNCTION organizations.validate_organization_constraints() OWNER TO living_landz_srv;

--
-- Name: update_zone_cell_count(); Type: FUNCTION; Schema: public; Owner: living_landz_srv
--

CREATE FUNCTION public.update_zone_cell_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$

BEGIN

    IF (TG_OP = 'INSERT') THEN

        UPDATE terrain.voronoi_zones

        SET cell_count = cell_count + 1,

            area_m2 = (cell_count + 1) * 50.0

        WHERE id = NEW.zone_id;

    ELSIF (TG_OP = 'DELETE') THEN

        UPDATE terrain.voronoi_zones

        SET cell_count = cell_count - 1,

            area_m2 = (cell_count - 1) * 50.0

        WHERE id = OLD.zone_id;

    END IF;

    RETURN NULL;

END;

$$;


ALTER FUNCTION public.update_zone_cell_count() OWNER TO living_landz_srv;

--
-- Name: update_item_decay(); Type: FUNCTION; Schema: resources; Owner: living_landz_srv
--

CREATE FUNCTION resources.update_item_decay() RETURNS trigger
    LANGUAGE plpgsql
    AS $$

DECLARE

    item_record RECORD;

    time_diff_seconds BIGINT;

    time_diff_days DECIMAL;

    decay_amount DECIMAL;

BEGIN

    -- R├®cup├®rer les infos de l'item

    SELECT is_perishable, base_decay_rate_per_day

    INTO item_record

    FROM resources.items

    WHERE id = NEW.item_id;



    -- Si l'item est p├®rissable et a un decay rate

    IF item_record.is_perishable AND item_record.base_decay_rate_per_day > 0 THEN

        -- Si c'est une nouvelle instance ou pas de derni├¿re mise ├á jour

        IF NEW.last_decay_update IS NULL THEN

            NEW.last_decay_update := NOW();

            NEW.current_decay := 0.0;

        ELSE

            -- Calculer le temps ├®coul├® en jours

            time_diff_seconds := EXTRACT(EPOCH FROM (NOW() - NEW.last_decay_update));

            time_diff_days := time_diff_seconds / 86400.0;



            -- Calculer le decay

            decay_amount := item_record.base_decay_rate_per_day * time_diff_days;

            NEW.current_decay := LEAST(1.0, NEW.current_decay + decay_amount);

            NEW.last_decay_update := NOW();

        END IF;

    END IF;



    RETURN NEW;

END;

$$;


ALTER FUNCTION resources.update_item_decay() OWNER TO living_landz_srv;

--
-- Name: FUNCTION update_item_decay(); Type: COMMENT; Schema: resources; Owner: living_landz_srv
--

COMMENT ON FUNCTION resources.update_item_decay() IS 'Automatically updates item decay based on time elapsed';


--
-- Name: cell_to_chunk(integer, integer); Type: FUNCTION; Schema: terrain; Owner: living_landz_srv
--

CREATE FUNCTION terrain.cell_to_chunk(cell_q integer, cell_r integer) RETURNS TABLE(chunk_x integer, chunk_y integer)
    LANGUAGE plpgsql IMMUTABLE
    AS $$

DECLARE

    hex_size FLOAT := 16.0;

    hex_ratio_x FLOAT := 1.0;

    hex_ratio_y FLOAT := 0.866025404;

    chunk_size_x FLOAT := 600.0;

    chunk_size_y FLOAT := 503.0;

    world_x FLOAT;

    world_y FLOAT;

BEGIN

    -- Conversion hex flat-top layout

    world_x := hex_size * hex_ratio_x * (1.5 * cell_q);

    world_y := hex_size * hex_ratio_y * (SQRT(3.0) * (cell_r + cell_q / 2.0));



    -- Calculer le chunk

    chunk_x := FLOOR(world_x / chunk_size_x)::INT;

    chunk_y := FLOOR(world_y / chunk_size_y)::INT;



    RETURN NEXT;

END;

$$;


ALTER FUNCTION terrain.cell_to_chunk(cell_q integer, cell_r integer) OWNER TO living_landz_srv;

--
-- Name: update_updated_at_column(); Type: FUNCTION; Schema: units; Owner: living_landz_srv
--

CREATE FUNCTION units.update_updated_at_column() RETURNS trigger
    LANGUAGE plpgsql
    AS $$

BEGIN

    NEW.updated_at = NOW();

    RETURN NEW;

END;

$$;


ALTER FUNCTION units.update_updated_at_column() OWNER TO living_landz_srv;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: action_specific_types; Type: TABLE; Schema: actions; Owner: living_landz_srv
--

CREATE TABLE actions.action_specific_types (
    id smallint NOT NULL,
    name character varying NOT NULL,
    archived boolean DEFAULT false
);


ALTER TABLE actions.action_specific_types OWNER TO living_landz_srv;

--
-- Name: action_statuses; Type: TABLE; Schema: actions; Owner: living_landz_srv
--

CREATE TABLE actions.action_statuses (
    id smallint NOT NULL,
    name character varying NOT NULL,
    archived boolean DEFAULT false
);


ALTER TABLE actions.action_statuses OWNER TO living_landz_srv;

--
-- Name: action_types; Type: TABLE; Schema: actions; Owner: living_landz_srv
--

CREATE TABLE actions.action_types (
    id smallint NOT NULL,
    name character varying NOT NULL,
    archived boolean DEFAULT false
);


ALTER TABLE actions.action_types OWNER TO living_landz_srv;

--
-- Name: build_building_actions; Type: TABLE; Schema: actions; Owner: living_landz_srv
--

CREATE TABLE actions.build_building_actions (
    action_id bigint NOT NULL,
    building_type_id integer NOT NULL
);


ALTER TABLE actions.build_building_actions OWNER TO living_landz_srv;

--
-- Name: build_road_actions; Type: TABLE; Schema: actions; Owner: living_landz_srv
--

CREATE TABLE actions.build_road_actions (
    action_id bigint NOT NULL,
    start_q integer NOT NULL,
    start_r integer NOT NULL,
    end_q integer NOT NULL,
    end_r integer NOT NULL
);


ALTER TABLE actions.build_road_actions OWNER TO living_landz_srv;

--
-- Name: craft_resource_actions; Type: TABLE; Schema: actions; Owner: living_landz_srv
--

CREATE TABLE actions.craft_resource_actions (
    action_id bigint NOT NULL,
    recipe_id character varying NOT NULL,
    quantity integer NOT NULL
);


ALTER TABLE actions.craft_resource_actions OWNER TO living_landz_srv;

--
-- Name: harvest_resource_actions; Type: TABLE; Schema: actions; Owner: living_landz_srv
--

CREATE TABLE actions.harvest_resource_actions (
    action_id bigint NOT NULL,
    resource_type_id integer NOT NULL
);


ALTER TABLE actions.harvest_resource_actions OWNER TO living_landz_srv;

--
-- Name: move_unit_actions; Type: TABLE; Schema: actions; Owner: living_landz_srv
--

CREATE TABLE actions.move_unit_actions (
    action_id bigint NOT NULL,
    unit_id bigint NOT NULL,
    target_q integer NOT NULL,
    target_r integer NOT NULL
);


ALTER TABLE actions.move_unit_actions OWNER TO living_landz_srv;

--
-- Name: scheduled_actions; Type: TABLE; Schema: actions; Owner: living_landz_srv
--

CREATE TABLE actions.scheduled_actions (
    id bigint NOT NULL,
    player_id bigint NOT NULL,
    cell_q integer NOT NULL,
    cell_r integer NOT NULL,
    chunk_x integer NOT NULL,
    chunk_y integer NOT NULL,
    action_type_id smallint NOT NULL,
    action_specific_type_id smallint NOT NULL,
    start_time bigint NOT NULL,
    duration_ms bigint NOT NULL,
    completion_time bigint NOT NULL,
    status_id smallint DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE actions.scheduled_actions OWNER TO living_landz_srv;

--
-- Name: scheduled_actions_id_seq; Type: SEQUENCE; Schema: actions; Owner: living_landz_srv
--

CREATE SEQUENCE actions.scheduled_actions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE actions.scheduled_actions_id_seq OWNER TO living_landz_srv;

--
-- Name: scheduled_actions_id_seq; Type: SEQUENCE OWNED BY; Schema: actions; Owner: living_landz_srv
--

ALTER SEQUENCE actions.scheduled_actions_id_seq OWNED BY actions.scheduled_actions.id;


--
-- Name: send_message_actions; Type: TABLE; Schema: actions; Owner: living_landz_srv
--

CREATE TABLE actions.send_message_actions (
    action_id bigint NOT NULL,
    message_content text NOT NULL
);


ALTER TABLE actions.send_message_actions OWNER TO living_landz_srv;

--
-- Name: send_message_receivers; Type: TABLE; Schema: actions; Owner: living_landz_srv
--

CREATE TABLE actions.send_message_receivers (
    id bigint NOT NULL,
    action_id bigint NOT NULL,
    receiver_id bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE actions.send_message_receivers OWNER TO living_landz_srv;

--
-- Name: send_message_receivers_id_seq; Type: SEQUENCE; Schema: actions; Owner: living_landz_srv
--

CREATE SEQUENCE actions.send_message_receivers_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE actions.send_message_receivers_id_seq OWNER TO living_landz_srv;

--
-- Name: send_message_receivers_id_seq; Type: SEQUENCE OWNED BY; Schema: actions; Owner: living_landz_srv
--

ALTER SEQUENCE actions.send_message_receivers_id_seq OWNED BY actions.send_message_receivers.id;


--
-- Name: train_unit_actions; Type: TABLE; Schema: actions; Owner: living_landz_srv
--

CREATE TABLE actions.train_unit_actions (
    action_id bigint NOT NULL,
    unit_id bigint NOT NULL,
    target_profession_id smallint NOT NULL
);


ALTER TABLE actions.train_unit_actions OWNER TO living_landz_srv;

--
-- Name: agriculture; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.agriculture (
    building_id bigint NOT NULL,
    agriculture_type_id smallint NOT NULL,
    variant integer DEFAULT 0 NOT NULL
);


ALTER TABLE buildings.agriculture OWNER TO living_landz_srv;

--
-- Name: agriculture_types; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.agriculture_types (
    id smallint NOT NULL,
    name character varying NOT NULL,
    archived boolean DEFAULT false
);


ALTER TABLE buildings.agriculture_types OWNER TO living_landz_srv;

--
-- Name: animal_breeding; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.animal_breeding (
    building_id bigint NOT NULL,
    animal_type_id smallint NOT NULL,
    variant integer DEFAULT 0 NOT NULL
);


ALTER TABLE buildings.animal_breeding OWNER TO living_landz_srv;

--
-- Name: animal_breeding_types; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.animal_breeding_types (
    id smallint NOT NULL,
    name character varying NOT NULL,
    archived boolean DEFAULT false
);


ALTER TABLE buildings.animal_breeding_types OWNER TO living_landz_srv;

--
-- Name: building_categories; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.building_categories (
    id smallint NOT NULL,
    name character varying NOT NULL,
    slug character varying(64)
);


ALTER TABLE buildings.building_categories OWNER TO living_landz_srv;

--
-- Name: building_specific_types; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.building_specific_types (
    id smallint NOT NULL,
    name character varying NOT NULL,
    archived boolean DEFAULT false,
    slug character varying(64)
);


ALTER TABLE buildings.building_specific_types OWNER TO living_landz_srv;

--
-- Name: building_types; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.building_types (
    id integer NOT NULL,
    name character varying NOT NULL,
    category_id smallint NOT NULL,
    specific_type_id smallint NOT NULL,
    description text,
    archived boolean DEFAULT false,
    slug character varying(64),
    construction_duration_seconds integer DEFAULT 15 NOT NULL
);


ALTER TABLE buildings.building_types OWNER TO living_landz_srv;

--
-- Name: building_types_id_seq; Type: SEQUENCE; Schema: buildings; Owner: living_landz_srv
--

CREATE SEQUENCE buildings.building_types_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE buildings.building_types_id_seq OWNER TO living_landz_srv;

--
-- Name: building_types_id_seq; Type: SEQUENCE OWNED BY; Schema: buildings; Owner: living_landz_srv
--

ALTER SEQUENCE buildings.building_types_id_seq OWNED BY buildings.building_types.id;


--
-- Name: buildings_base; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.buildings_base (
    id bigint NOT NULL,
    building_type_id integer NOT NULL,
    category_id smallint NOT NULL,
    chunk_x integer NOT NULL,
    chunk_y integer NOT NULL,
    cell_q integer NOT NULL,
    cell_r integer NOT NULL,
    quality double precision DEFAULT 1.0 NOT NULL,
    durability double precision DEFAULT 1.0 NOT NULL,
    damage double precision DEFAULT 0.0 NOT NULL,
    is_built boolean DEFAULT true NOT NULL,
    created_at bigint NOT NULL
);


ALTER TABLE buildings.buildings_base OWNER TO living_landz_srv;

--
-- Name: buildings_base_id_seq; Type: SEQUENCE; Schema: buildings; Owner: living_landz_srv
--

CREATE SEQUENCE buildings.buildings_base_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE buildings.buildings_base_id_seq OWNER TO living_landz_srv;

--
-- Name: buildings_base_id_seq; Type: SEQUENCE OWNED BY; Schema: buildings; Owner: living_landz_srv
--

ALTER SEQUENCE buildings.buildings_base_id_seq OWNED BY buildings.buildings_base.id;


--
-- Name: commerce; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.commerce (
    building_id bigint NOT NULL,
    commerce_type_id smallint NOT NULL,
    variant integer DEFAULT 0 NOT NULL
);


ALTER TABLE buildings.commerce OWNER TO living_landz_srv;

--
-- Name: commerce_types; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.commerce_types (
    id smallint NOT NULL,
    name character varying NOT NULL,
    archived boolean DEFAULT false
);


ALTER TABLE buildings.commerce_types OWNER TO living_landz_srv;

--
-- Name: construction_costs; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.construction_costs (
    building_type_id integer NOT NULL,
    item_id integer NOT NULL,
    quantity integer NOT NULL,
    CONSTRAINT chk_construction_cost_qty CHECK ((quantity > 0))
);


ALTER TABLE buildings.construction_costs OWNER TO living_landz_srv;

--
-- Name: cult; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.cult (
    building_id bigint NOT NULL,
    cult_type_id smallint NOT NULL,
    variant integer DEFAULT 0 NOT NULL
);


ALTER TABLE buildings.cult OWNER TO living_landz_srv;

--
-- Name: cult_types; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.cult_types (
    id smallint NOT NULL,
    name character varying NOT NULL,
    archived boolean DEFAULT false
);


ALTER TABLE buildings.cult_types OWNER TO living_landz_srv;

--
-- Name: entertainment; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.entertainment (
    building_id bigint NOT NULL,
    entertainment_type_id smallint NOT NULL,
    variant integer DEFAULT 0 NOT NULL
);


ALTER TABLE buildings.entertainment OWNER TO living_landz_srv;

--
-- Name: entertainment_types; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.entertainment_types (
    id smallint NOT NULL,
    name character varying NOT NULL,
    archived boolean DEFAULT false
);


ALTER TABLE buildings.entertainment_types OWNER TO living_landz_srv;

--
-- Name: manufacturing_workshop_types; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.manufacturing_workshop_types (
    id smallint NOT NULL,
    name character varying NOT NULL,
    archived boolean DEFAULT false
);


ALTER TABLE buildings.manufacturing_workshop_types OWNER TO living_landz_srv;

--
-- Name: manufacturing_workshops; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.manufacturing_workshops (
    building_id bigint NOT NULL,
    workshop_type_id smallint NOT NULL,
    variant integer DEFAULT 0 NOT NULL
);


ALTER TABLE buildings.manufacturing_workshops OWNER TO living_landz_srv;

--
-- Name: tree_types; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.tree_types (
    id smallint NOT NULL,
    name character varying NOT NULL,
    archived boolean DEFAULT false
);


ALTER TABLE buildings.tree_types OWNER TO living_landz_srv;

--
-- Name: trees; Type: TABLE; Schema: buildings; Owner: living_landz_srv
--

CREATE TABLE buildings.trees (
    building_id bigint NOT NULL,
    tree_type_id smallint NOT NULL,
    density integer NOT NULL,
    age integer NOT NULL,
    variant integer NOT NULL
);


ALTER TABLE buildings.trees OWNER TO living_landz_srv;

--
-- Name: characters; Type: TABLE; Schema: game; Owner: living_landz_srv
--

CREATE TABLE game.characters (
    id bigint NOT NULL,
    player_id bigint NOT NULL,
    first_name character varying NOT NULL,
    family_name character varying NOT NULL,
    second_name character varying,
    nickname character varying,
    coat_of_arms_id bigint,
    image_id bigint,
    motto character varying,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);


ALTER TABLE game.characters OWNER TO living_landz_srv;

--
-- Name: TABLE characters; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON TABLE game.characters IS 'Personnages individuels des joueurs';


--
-- Name: COLUMN characters.player_id; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON COLUMN game.characters.player_id IS 'Joueur propri├®taire du personnage';


--
-- Name: COLUMN characters.family_name; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON COLUMN game.characters.family_name IS 'Nom de famille d''usage (peut diff├®rer du nom de famille du joueur)';


--
-- Name: COLUMN characters.second_name; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON COLUMN game.characters.second_name IS 'Deuxi├¿me pr├®nom ou nom interm├®diaire';


--
-- Name: COLUMN characters.coat_of_arms_id; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON COLUMN game.characters.coat_of_arms_id IS 'Armoiries personnelles du caract├¿re';


--
-- Name: COLUMN characters.image_id; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON COLUMN game.characters.image_id IS 'R├®f├®rence ├á une image du personnage';


--
-- Name: COLUMN characters.motto; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON COLUMN game.characters.motto IS 'Devise personnelle du personnage';


--
-- Name: characters_id_seq; Type: SEQUENCE; Schema: game; Owner: living_landz_srv
--

CREATE SEQUENCE game.characters_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE game.characters_id_seq OWNER TO living_landz_srv;

--
-- Name: characters_id_seq; Type: SEQUENCE OWNED BY; Schema: game; Owner: living_landz_srv
--

ALTER SEQUENCE game.characters_id_seq OWNED BY game.characters.id;


--
-- Name: coats_of_arms; Type: TABLE; Schema: game; Owner: living_landz_srv
--

CREATE TABLE game.coats_of_arms (
    id bigint NOT NULL,
    name character varying NOT NULL,
    description text,
    image_data bytea,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE game.coats_of_arms OWNER TO living_landz_srv;

--
-- Name: coats_of_arms_id_seq; Type: SEQUENCE; Schema: game; Owner: living_landz_srv
--

CREATE SEQUENCE game.coats_of_arms_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE game.coats_of_arms_id_seq OWNER TO living_landz_srv;

--
-- Name: coats_of_arms_id_seq; Type: SEQUENCE OWNED BY; Schema: game; Owner: living_landz_srv
--

ALTER SEQUENCE game.coats_of_arms_id_seq OWNED BY game.coats_of_arms.id;


--
-- Name: languages; Type: TABLE; Schema: game; Owner: living_landz_srv
--

CREATE TABLE game.languages (
    id smallint NOT NULL,
    name character varying NOT NULL,
    code character varying(5) NOT NULL
);


ALTER TABLE game.languages OWNER TO living_landz_srv;

--
-- Name: players; Type: TABLE; Schema: game; Owner: living_landz_srv
--

CREATE TABLE game.players (
    id bigint NOT NULL,
    family_name character varying NOT NULL,
    language_id smallint NOT NULL,
    coat_of_arms_id bigint,
    motto character varying,
    origin_location character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    password_hash character varying(255),
    email character varying(255),
    account_status character varying(20) DEFAULT 'active'::character varying,
    last_login_at timestamp with time zone,
    CONSTRAINT players_account_status_check CHECK (((account_status)::text = ANY ((ARRAY['active'::character varying, 'locked'::character varying, 'suspended'::character varying])::text[])))
);


ALTER TABLE game.players OWNER TO living_landz_srv;

--
-- Name: TABLE players; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON TABLE game.players IS 'Joueurs principaux du jeu';


--
-- Name: COLUMN players.family_name; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON COLUMN game.players.family_name IS 'Nom de famille du joueur (nom de la maison/dynastie)';


--
-- Name: COLUMN players.language_id; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON COLUMN game.players.language_id IS 'Langue d''origine du joueur';


--
-- Name: COLUMN players.coat_of_arms_id; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON COLUMN game.players.coat_of_arms_id IS 'Armoiries principales de la maison';


--
-- Name: COLUMN players.motto; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON COLUMN game.players.motto IS 'Devise de la maison';


--
-- Name: COLUMN players.origin_location; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON COLUMN game.players.origin_location IS 'Lieu d''origine de la maison';


--
-- Name: COLUMN players.password_hash; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON COLUMN game.players.password_hash IS 'Argon2id password hash in PHC format (includes salt)';


--
-- Name: COLUMN players.email; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON COLUMN game.players.email IS 'Email address for password recovery (optional)';


--
-- Name: COLUMN players.account_status; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON COLUMN game.players.account_status IS 'Account status: active, locked, or suspended';


--
-- Name: COLUMN players.last_login_at; Type: COMMENT; Schema: game; Owner: living_landz_srv
--

COMMENT ON COLUMN game.players.last_login_at IS 'Timestamp of last successful login';


--
-- Name: players_id_seq; Type: SEQUENCE; Schema: game; Owner: living_landz_srv
--

CREATE SEQUENCE game.players_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE game.players_id_seq OWNER TO living_landz_srv;

--
-- Name: players_id_seq; Type: SEQUENCE OWNED BY; Schema: game; Owner: living_landz_srv
--

ALTER SEQUENCE game.players_id_seq OWNED BY game.players.id;


--
-- Name: translations; Type: TABLE; Schema: game; Owner: living_landz_srv
--

CREATE TABLE game.translations (
    entity_type character varying(50) NOT NULL,
    entity_id integer NOT NULL,
    language_id smallint NOT NULL,
    field character varying(50) NOT NULL,
    value text NOT NULL
);


ALTER TABLE game.translations OWNER TO living_landz_srv;

--
-- Name: buildings; Type: TABLE; Schema: organizations; Owner: living_landz_srv
--

CREATE TABLE organizations.buildings (
    id bigint NOT NULL,
    organization_id bigint NOT NULL,
    building_id bigint NOT NULL,
    acquired_at timestamp with time zone DEFAULT now(),
    acquired_by_unit_id bigint,
    building_role character varying(50)
);


ALTER TABLE organizations.buildings OWNER TO living_landz_srv;

--
-- Name: TABLE buildings; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON TABLE organizations.buildings IS 'B├ótiments poss├®d├®s par les organisations';


--
-- Name: COLUMN buildings.acquired_by_unit_id; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.buildings.acquired_by_unit_id IS 'ID de l''unit├® qui a acquis ce b├ótiment';


--
-- Name: COLUMN buildings.building_role; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.buildings.building_role IS 'R├┤le du b├ótiment: headquarters, warehouse, barracks, etc.';


--
-- Name: buildings_id_seq; Type: SEQUENCE; Schema: organizations; Owner: living_landz_srv
--

CREATE SEQUENCE organizations.buildings_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE organizations.buildings_id_seq OWNER TO living_landz_srv;

--
-- Name: buildings_id_seq; Type: SEQUENCE OWNED BY; Schema: organizations; Owner: living_landz_srv
--

ALTER SEQUENCE organizations.buildings_id_seq OWNED BY organizations.buildings.id;


--
-- Name: diplomatic_relations; Type: TABLE; Schema: organizations; Owner: living_landz_srv
--

CREATE TABLE organizations.diplomatic_relations (
    id bigint NOT NULL,
    organization_id bigint NOT NULL,
    target_organization_id bigint NOT NULL,
    relation_type character varying(20) NOT NULL,
    established_at timestamp with time zone DEFAULT now(),
    established_by_unit_id bigint,
    expires_at timestamp with time zone,
    CONSTRAINT diplomatic_relations_check CHECK ((organization_id <> target_organization_id)),
    CONSTRAINT diplomatic_relations_relation_type_check CHECK (((relation_type)::text = ANY ((ARRAY['allied'::character varying, 'neutral'::character varying, 'hostile'::character varying, 'at_war'::character varying, 'trade_agreement'::character varying, 'non_aggression'::character varying])::text[])))
);


ALTER TABLE organizations.diplomatic_relations OWNER TO living_landz_srv;

--
-- Name: TABLE diplomatic_relations; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON TABLE organizations.diplomatic_relations IS 'Relations diplomatiques entre organisations';


--
-- Name: COLUMN diplomatic_relations.relation_type; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.diplomatic_relations.relation_type IS 'Type: allied, neutral, hostile, at_war, trade_agreement, non_aggression';


--
-- Name: COLUMN diplomatic_relations.expires_at; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.diplomatic_relations.expires_at IS 'Date d''expiration (NULL = permanent)';


--
-- Name: diplomatic_relations_id_seq; Type: SEQUENCE; Schema: organizations; Owner: living_landz_srv
--

CREATE SEQUENCE organizations.diplomatic_relations_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE organizations.diplomatic_relations_id_seq OWNER TO living_landz_srv;

--
-- Name: diplomatic_relations_id_seq; Type: SEQUENCE OWNED BY; Schema: organizations; Owner: living_landz_srv
--

ALTER SEQUENCE organizations.diplomatic_relations_id_seq OWNED BY organizations.diplomatic_relations.id;


--
-- Name: members; Type: TABLE; Schema: organizations; Owner: living_landz_srv
--

CREATE TABLE organizations.members (
    id bigint NOT NULL,
    organization_id bigint NOT NULL,
    unit_id bigint NOT NULL,
    joined_at timestamp with time zone DEFAULT now(),
    invited_by_unit_id bigint,
    membership_status character varying(20) DEFAULT 'active'::character varying,
    CONSTRAINT members_membership_status_check CHECK (((membership_status)::text = ANY ((ARRAY['active'::character varying, 'suspended'::character varying, 'honorary'::character varying])::text[])))
);


ALTER TABLE organizations.members OWNER TO living_landz_srv;

--
-- Name: TABLE members; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON TABLE organizations.members IS 'Membres des organisations';


--
-- Name: COLUMN members.invited_by_unit_id; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.members.invited_by_unit_id IS 'ID de l''unit├® qui a invit├® ce membre';


--
-- Name: COLUMN members.membership_status; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.members.membership_status IS 'Statut: active, suspended, honorary';


--
-- Name: members_id_seq; Type: SEQUENCE; Schema: organizations; Owner: living_landz_srv
--

CREATE SEQUENCE organizations.members_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE organizations.members_id_seq OWNER TO living_landz_srv;

--
-- Name: members_id_seq; Type: SEQUENCE OWNED BY; Schema: organizations; Owner: living_landz_srv
--

ALTER SEQUENCE organizations.members_id_seq OWNED BY organizations.members.id;


--
-- Name: officers; Type: TABLE; Schema: organizations; Owner: living_landz_srv
--

CREATE TABLE organizations.officers (
    id bigint NOT NULL,
    organization_id bigint NOT NULL,
    unit_id bigint NOT NULL,
    role_type_id smallint NOT NULL,
    appointed_at timestamp with time zone DEFAULT now(),
    appointed_by_unit_id bigint
);


ALTER TABLE organizations.officers OWNER TO living_landz_srv;

--
-- Name: TABLE officers; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON TABLE organizations.officers IS 'Officiers et postes importants dans les organisations';


--
-- Name: COLUMN officers.appointed_by_unit_id; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.officers.appointed_by_unit_id IS 'ID de l''unit├® qui a nomm├® cet officier';


--
-- Name: officers_id_seq; Type: SEQUENCE; Schema: organizations; Owner: living_landz_srv
--

CREATE SEQUENCE organizations.officers_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE organizations.officers_id_seq OWNER TO living_landz_srv;

--
-- Name: officers_id_seq; Type: SEQUENCE OWNED BY; Schema: organizations; Owner: living_landz_srv
--

ALTER SEQUENCE organizations.officers_id_seq OWNED BY organizations.officers.id;


--
-- Name: organization_role_compatibility; Type: TABLE; Schema: organizations; Owner: living_landz_srv
--

CREATE TABLE organizations.organization_role_compatibility (
    organization_type_id smallint NOT NULL,
    role_type_id smallint NOT NULL,
    is_leader_role boolean DEFAULT false NOT NULL
);


ALTER TABLE organizations.organization_role_compatibility OWNER TO living_landz_srv;

--
-- Name: TABLE organization_role_compatibility; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON TABLE organizations.organization_role_compatibility IS 'D├®finit quels r├┤les sont compatibles avec quels types d''organisation';


--
-- Name: organization_types; Type: TABLE; Schema: organizations; Owner: living_landz_srv
--

CREATE TABLE organizations.organization_types (
    id smallint NOT NULL,
    name character varying(50) NOT NULL,
    category character varying(20) NOT NULL,
    requires_territory boolean DEFAULT true NOT NULL,
    can_have_vassals boolean DEFAULT false NOT NULL,
    can_have_parent boolean DEFAULT false NOT NULL,
    min_population integer,
    min_area_km2 numeric(10,2),
    description text
);


ALTER TABLE organizations.organization_types OWNER TO living_landz_srv;

--
-- Name: TABLE organization_types; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON TABLE organizations.organization_types IS 'Types d''organisations disponibles';


--
-- Name: COLUMN organization_types.category; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.organization_types.category IS 'Cat├®gorie: territorial, religious, commercial, social';


--
-- Name: COLUMN organization_types.requires_territory; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.organization_types.requires_territory IS 'N├®cessite un territoire pour exister';


--
-- Name: COLUMN organization_types.can_have_vassals; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.organization_types.can_have_vassals IS 'Peut avoir des vassaux/organisations subordonn├®es';


--
-- Name: COLUMN organization_types.can_have_parent; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.organization_types.can_have_parent IS 'Peut avoir une organisation parente (vassalit├®)';


--
-- Name: organizations; Type: TABLE; Schema: organizations; Owner: living_landz_srv
--

CREATE TABLE organizations.organizations (
    id bigint NOT NULL,
    name character varying(100) NOT NULL,
    organization_type_id smallint NOT NULL,
    parent_organization_id bigint,
    headquarters_cell_q integer,
    headquarters_cell_r integer,
    total_area_km2 numeric(10,2) DEFAULT 0,
    treasury_gold integer DEFAULT 0,
    leader_unit_id bigint,
    emblem_url character varying(255),
    population integer DEFAULT 0,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    voronoi_zone_id bigint,
    CONSTRAINT organizations_population_check CHECK ((population >= 0)),
    CONSTRAINT organizations_treasury_gold_check CHECK ((treasury_gold >= 0)),
    CONSTRAINT valid_headquarters CHECK ((((headquarters_cell_q IS NULL) AND (headquarters_cell_r IS NULL)) OR ((headquarters_cell_q IS NOT NULL) AND (headquarters_cell_r IS NOT NULL))))
);


ALTER TABLE organizations.organizations OWNER TO living_landz_srv;

--
-- Name: TABLE organizations; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON TABLE organizations.organizations IS 'Table principale des organisations (villes, royaumes, guildes, etc.)';


--
-- Name: COLUMN organizations.parent_organization_id; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.organizations.parent_organization_id IS 'Organisation parente (vassalit├®)';


--
-- Name: COLUMN organizations.headquarters_cell_q; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.organizations.headquarters_cell_q IS 'Coordonn├®e Q de la cellule du si├¿ge';


--
-- Name: COLUMN organizations.total_area_km2; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.organizations.total_area_km2 IS 'Superficie totale en km┬▓ (calcul├®e automatiquement)';


--
-- Name: COLUMN organizations.treasury_gold; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.organizations.treasury_gold IS 'Or dans la tr├®sorerie';


--
-- Name: COLUMN organizations.population; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.organizations.population IS 'Population totale (calcul├®e automatiquement)';


--
-- Name: organizations_id_seq; Type: SEQUENCE; Schema: organizations; Owner: living_landz_srv
--

CREATE SEQUENCE organizations.organizations_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE organizations.organizations_id_seq OWNER TO living_landz_srv;

--
-- Name: organizations_id_seq; Type: SEQUENCE OWNED BY; Schema: organizations; Owner: living_landz_srv
--

ALTER SEQUENCE organizations.organizations_id_seq OWNED BY organizations.organizations.id;


--
-- Name: role_types; Type: TABLE; Schema: organizations; Owner: living_landz_srv
--

CREATE TABLE organizations.role_types (
    id smallint NOT NULL,
    name character varying(50) NOT NULL,
    category character varying(20) NOT NULL,
    authority_level smallint NOT NULL,
    description text
);


ALTER TABLE organizations.role_types OWNER TO living_landz_srv;

--
-- Name: TABLE role_types; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON TABLE organizations.role_types IS 'R├┤les/postes disponibles dans les organisations';


--
-- Name: COLUMN role_types.authority_level; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.role_types.authority_level IS 'Niveau d''autorit├®: 1=le plus ├®lev├®, 100=le plus bas';


--
-- Name: territory_cells; Type: TABLE; Schema: organizations; Owner: living_landz_srv
--

CREATE TABLE organizations.territory_cells (
    organization_id bigint NOT NULL,
    cell_q integer NOT NULL,
    cell_r integer NOT NULL,
    claimed_at timestamp with time zone DEFAULT now()
);


ALTER TABLE organizations.territory_cells OWNER TO living_landz_srv;

--
-- Name: TABLE territory_cells; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON TABLE organizations.territory_cells IS 'Cellules de territoire contr├┤l├®es par les organisations';


--
-- Name: COLUMN territory_cells.claimed_at; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.territory_cells.claimed_at IS 'Date de revendication du territoire';


--
-- Name: territory_contours; Type: TABLE; Schema: organizations; Owner: living_landz_srv
--

CREATE TABLE organizations.territory_contours (
    id bigint NOT NULL,
    organization_id bigint NOT NULL,
    chunk_x integer NOT NULL,
    chunk_y integer NOT NULL,
    contour_segments real[] NOT NULL,
    bbox_min_x real NOT NULL,
    bbox_min_y real NOT NULL,
    bbox_max_x real NOT NULL,
    bbox_max_y real NOT NULL,
    segment_count integer NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE organizations.territory_contours OWNER TO living_landz_srv;

--
-- Name: TABLE territory_contours; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON TABLE organizations.territory_contours IS 'Stores territory contour points per organization per chunk for client rendering';


--
-- Name: COLUMN territory_contours.contour_segments; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.territory_contours.contour_segments IS 'Flattened array of segment data [start.x,start.y,end.x,end.y,normal.x,normal.y,...] where each segment has 6 floats';


--
-- Name: COLUMN territory_contours.bbox_min_x; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.territory_contours.bbox_min_x IS 'Minimum X coordinate of bounding box';


--
-- Name: COLUMN territory_contours.segment_count; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.territory_contours.segment_count IS 'Number of ContourSegment (length of contour_segments / 6)';


--
-- Name: territory_contours_id_seq; Type: SEQUENCE; Schema: organizations; Owner: living_landz_srv
--

CREATE SEQUENCE organizations.territory_contours_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE organizations.territory_contours_id_seq OWNER TO living_landz_srv;

--
-- Name: territory_contours_id_seq; Type: SEQUENCE OWNED BY; Schema: organizations; Owner: living_landz_srv
--

ALTER SEQUENCE organizations.territory_contours_id_seq OWNED BY organizations.territory_contours.id;


--
-- Name: treasury_items; Type: TABLE; Schema: organizations; Owner: living_landz_srv
--

CREATE TABLE organizations.treasury_items (
    id bigint NOT NULL,
    organization_id bigint NOT NULL,
    item_instance_id bigint NOT NULL,
    quantity integer DEFAULT 1 NOT NULL,
    stored_at timestamp with time zone DEFAULT now(),
    CONSTRAINT treasury_items_quantity_check CHECK ((quantity > 0))
);


ALTER TABLE organizations.treasury_items OWNER TO living_landz_srv;

--
-- Name: TABLE treasury_items; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON TABLE organizations.treasury_items IS 'Inventaire/tr├®sorerie des organisations';


--
-- Name: COLUMN treasury_items.quantity; Type: COMMENT; Schema: organizations; Owner: living_landz_srv
--

COMMENT ON COLUMN organizations.treasury_items.quantity IS 'Quantit├® d''items (pour items stackables)';


--
-- Name: treasury_items_id_seq; Type: SEQUENCE; Schema: organizations; Owner: living_landz_srv
--

CREATE SEQUENCE organizations.treasury_items_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE organizations.treasury_items_id_seq OWNER TO living_landz_srv;

--
-- Name: treasury_items_id_seq; Type: SEQUENCE OWNED BY; Schema: organizations; Owner: living_landz_srv
--

ALTER SEQUENCE organizations.treasury_items_id_seq OWNED BY organizations.treasury_items.id;


--
-- Name: _sqlx_migrations; Type: TABLE; Schema: public; Owner: living_landz_srv
--

CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);


ALTER TABLE public._sqlx_migrations OWNER TO living_landz_srv;

--
-- Name: equipment_slots; Type: TABLE; Schema: resources; Owner: living_landz_srv
--

CREATE TABLE resources.equipment_slots (
    id smallint NOT NULL,
    name character varying NOT NULL,
    archived boolean DEFAULT false,
    slug character varying(64)
);


ALTER TABLE resources.equipment_slots OWNER TO living_landz_srv;

--
-- Name: harvest_yields; Type: TABLE; Schema: resources; Owner: living_landz_srv
--

CREATE TABLE resources.harvest_yields (
    id integer NOT NULL,
    resource_specific_type_id smallint NOT NULL,
    result_item_id integer NOT NULL,
    base_quantity integer DEFAULT 1 NOT NULL,
    quality_min numeric(3,2) DEFAULT 0.50 NOT NULL,
    quality_max numeric(3,2) DEFAULT 1.00 NOT NULL,
    required_profession_id smallint,
    required_tool_item_id integer,
    tool_bonus_quantity integer DEFAULT 0,
    duration_seconds integer DEFAULT 30 NOT NULL,
    CONSTRAINT chk_harvest_base_qty CHECK ((base_quantity > 0)),
    CONSTRAINT chk_harvest_quality CHECK ((quality_min <= quality_max))
);


ALTER TABLE resources.harvest_yields OWNER TO living_landz_srv;

--
-- Name: harvest_yields_id_seq; Type: SEQUENCE; Schema: resources; Owner: living_landz_srv
--

CREATE SEQUENCE resources.harvest_yields_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE resources.harvest_yields_id_seq OWNER TO living_landz_srv;

--
-- Name: harvest_yields_id_seq; Type: SEQUENCE OWNED BY; Schema: resources; Owner: living_landz_srv
--

ALTER SEQUENCE resources.harvest_yields_id_seq OWNED BY resources.harvest_yields.id;


--
-- Name: item_instances; Type: TABLE; Schema: resources; Owner: living_landz_srv
--

CREATE TABLE resources.item_instances (
    id bigint NOT NULL,
    item_id integer NOT NULL,
    quality numeric(3,2) DEFAULT 1.0 NOT NULL,
    current_decay numeric(3,2) DEFAULT 0.0,
    last_decay_update timestamp with time zone,
    owner_unit_id bigint,
    world_cell_q integer,
    world_cell_r integer,
    world_chunk_x integer,
    world_chunk_y integer,
    created_at timestamp with time zone DEFAULT now(),
    CONSTRAINT chk_current_decay CHECK (((current_decay >= 0.0) AND (current_decay <= 1.0))),
    CONSTRAINT chk_quality CHECK (((quality >= 0.0) AND (quality <= 1.0)))
);


ALTER TABLE resources.item_instances OWNER TO living_landz_srv;

--
-- Name: TABLE item_instances; Type: COMMENT; Schema: resources; Owner: living_landz_srv
--

COMMENT ON TABLE resources.item_instances IS 'Individual item instances with quality and decay tracking';


--
-- Name: COLUMN item_instances.quality; Type: COMMENT; Schema: resources; Owner: living_landz_srv
--

COMMENT ON COLUMN resources.item_instances.quality IS 'Quality of the item, 0.0 to 1.0, affects stats and price';


--
-- Name: COLUMN item_instances.current_decay; Type: COMMENT; Schema: resources; Owner: living_landz_srv
--

COMMENT ON COLUMN resources.item_instances.current_decay IS 'Current decay level, 0.0 = fresh, 1.0 = completely rotten';


--
-- Name: COLUMN item_instances.owner_unit_id; Type: COMMENT; Schema: resources; Owner: living_landz_srv
--

COMMENT ON COLUMN resources.item_instances.owner_unit_id IS 'Unit that owns this item, NULL if in the world';


--
-- Name: item_instances_id_seq; Type: SEQUENCE; Schema: resources; Owner: living_landz_srv
--

CREATE SEQUENCE resources.item_instances_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE resources.item_instances_id_seq OWNER TO living_landz_srv;

--
-- Name: item_instances_id_seq; Type: SEQUENCE OWNED BY; Schema: resources; Owner: living_landz_srv
--

ALTER SEQUENCE resources.item_instances_id_seq OWNED BY resources.item_instances.id;


--
-- Name: item_stat_modifiers; Type: TABLE; Schema: resources; Owner: living_landz_srv
--

CREATE TABLE resources.item_stat_modifiers (
    item_id integer NOT NULL,
    stat_name character varying NOT NULL,
    modifier_value integer NOT NULL
);


ALTER TABLE resources.item_stat_modifiers OWNER TO living_landz_srv;

--
-- Name: TABLE item_stat_modifiers; Type: COMMENT; Schema: resources; Owner: living_landz_srv
--

COMMENT ON TABLE resources.item_stat_modifiers IS 'Stat bonuses provided by items when equipped or used';


--
-- Name: item_types; Type: TABLE; Schema: resources; Owner: living_landz_srv
--

CREATE TABLE resources.item_types (
    id smallint NOT NULL,
    name character varying NOT NULL,
    archived boolean DEFAULT false,
    slug character varying(64)
);


ALTER TABLE resources.item_types OWNER TO living_landz_srv;

--
-- Name: items; Type: TABLE; Schema: resources; Owner: living_landz_srv
--

CREATE TABLE resources.items (
    id integer NOT NULL,
    name character varying NOT NULL,
    item_type_id smallint NOT NULL,
    category_id smallint,
    weight_kg numeric(10,3) DEFAULT 0.001 NOT NULL,
    volume_liters numeric(10,3) DEFAULT 0.001 NOT NULL,
    base_price integer DEFAULT 0 NOT NULL,
    is_perishable boolean DEFAULT false,
    base_decay_rate_per_day numeric(5,4) DEFAULT 0,
    is_equipable boolean DEFAULT false,
    equipment_slot_id smallint,
    is_craftable boolean DEFAULT false,
    description text,
    image_url character varying(500),
    archived boolean DEFAULT false,
    created_at timestamp with time zone DEFAULT now(),
    slug character varying(64)
);


ALTER TABLE resources.items OWNER TO living_landz_srv;

--
-- Name: TABLE items; Type: COMMENT; Schema: resources; Owner: living_landz_srv
--

COMMENT ON TABLE resources.items IS 'Item definitions - base templates for all items in the game';


--
-- Name: COLUMN items.base_price; Type: COMMENT; Schema: resources; Owner: living_landz_srv
--

COMMENT ON COLUMN resources.items.base_price IS 'Base price in copper coins (100 copper = 1 silver, 100 silver = 1 gold)';


--
-- Name: COLUMN items.base_decay_rate_per_day; Type: COMMENT; Schema: resources; Owner: living_landz_srv
--

COMMENT ON COLUMN resources.items.base_decay_rate_per_day IS 'Base decay rate per day for perishable items (0.0 = no decay, 1.0 = complete decay in 1 day)';


--
-- Name: items_id_seq; Type: SEQUENCE; Schema: resources; Owner: living_landz_srv
--

CREATE SEQUENCE resources.items_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE resources.items_id_seq OWNER TO living_landz_srv;

--
-- Name: items_id_seq; Type: SEQUENCE OWNED BY; Schema: resources; Owner: living_landz_srv
--

ALTER SEQUENCE resources.items_id_seq OWNED BY resources.items.id;


--
-- Name: recipe_ingredients; Type: TABLE; Schema: resources; Owner: living_landz_srv
--

CREATE TABLE resources.recipe_ingredients (
    recipe_id integer NOT NULL,
    item_id integer NOT NULL,
    quantity integer NOT NULL,
    CONSTRAINT chk_ingredient_quantity CHECK ((quantity > 0))
);


ALTER TABLE resources.recipe_ingredients OWNER TO living_landz_srv;

--
-- Name: TABLE recipe_ingredients; Type: COMMENT; Schema: resources; Owner: living_landz_srv
--

COMMENT ON TABLE resources.recipe_ingredients IS 'Ingredients required for each recipe';


--
-- Name: recipes; Type: TABLE; Schema: resources; Owner: living_landz_srv
--

CREATE TABLE resources.recipes (
    id integer NOT NULL,
    name character varying NOT NULL,
    description text,
    result_item_id integer NOT NULL,
    result_quantity integer DEFAULT 1 NOT NULL,
    required_skill_id smallint,
    required_skill_level integer DEFAULT 1,
    craft_duration_seconds integer DEFAULT 10 NOT NULL,
    required_building_type_id smallint,
    archived boolean DEFAULT false,
    created_at timestamp with time zone DEFAULT now(),
    slug character varying(64),
    CONSTRAINT chk_craft_duration CHECK ((craft_duration_seconds > 0)),
    CONSTRAINT chk_result_quantity CHECK ((result_quantity > 0))
);


ALTER TABLE resources.recipes OWNER TO living_landz_srv;

--
-- Name: TABLE recipes; Type: COMMENT; Schema: resources; Owner: living_landz_srv
--

COMMENT ON TABLE resources.recipes IS 'Crafting recipes for creating items';


--
-- Name: COLUMN recipes.required_building_type_id; Type: COMMENT; Schema: resources; Owner: living_landz_srv
--

COMMENT ON COLUMN resources.recipes.required_building_type_id IS 'Building type ID from buildings.building_types, NULL if can craft anywhere';


--
-- Name: recipes_id_seq; Type: SEQUENCE; Schema: resources; Owner: living_landz_srv
--

CREATE SEQUENCE resources.recipes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE resources.recipes_id_seq OWNER TO living_landz_srv;

--
-- Name: recipes_id_seq; Type: SEQUENCE OWNED BY; Schema: resources; Owner: living_landz_srv
--

ALTER SEQUENCE resources.recipes_id_seq OWNED BY resources.recipes.id;


--
-- Name: resource_categories; Type: TABLE; Schema: resources; Owner: living_landz_srv
--

CREATE TABLE resources.resource_categories (
    id smallint NOT NULL,
    name character varying NOT NULL,
    slug character varying(64)
);


ALTER TABLE resources.resource_categories OWNER TO living_landz_srv;

--
-- Name: resource_specific_types; Type: TABLE; Schema: resources; Owner: living_landz_srv
--

CREATE TABLE resources.resource_specific_types (
    id smallint NOT NULL,
    name character varying NOT NULL,
    category_id smallint NOT NULL,
    archived boolean DEFAULT false,
    slug character varying(64)
);


ALTER TABLE resources.resource_specific_types OWNER TO living_landz_srv;

--
-- Name: resource_types; Type: TABLE; Schema: resources; Owner: living_landz_srv
--

CREATE TABLE resources.resource_types (
    id integer NOT NULL,
    name character varying NOT NULL,
    category_id smallint NOT NULL,
    specific_type_id smallint NOT NULL,
    description text,
    archived boolean DEFAULT false
);


ALTER TABLE resources.resource_types OWNER TO living_landz_srv;

--
-- Name: resource_types_id_seq; Type: SEQUENCE; Schema: resources; Owner: living_landz_srv
--

CREATE SEQUENCE resources.resource_types_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE resources.resource_types_id_seq OWNER TO living_landz_srv;

--
-- Name: resource_types_id_seq; Type: SEQUENCE OWNED BY; Schema: resources; Owner: living_landz_srv
--

ALTER SEQUENCE resources.resource_types_id_seq OWNED BY resources.resource_types.id;


--
-- Name: biome_types; Type: TABLE; Schema: terrain; Owner: living_landz_srv
--

CREATE TABLE terrain.biome_types (
    id smallint NOT NULL,
    name character varying NOT NULL
);


ALTER TABLE terrain.biome_types OWNER TO living_landz_srv;

--
-- Name: cells; Type: TABLE; Schema: terrain; Owner: living_landz_srv
--

CREATE TABLE terrain.cells (
    q integer NOT NULL,
    r integer NOT NULL,
    biome_id smallint NOT NULL,
    terrain_type character varying,
    building_id bigint,
    chunk_x integer NOT NULL,
    chunk_y integer NOT NULL
);


ALTER TABLE terrain.cells OWNER TO living_landz_srv;

--
-- Name: ocean_data; Type: TABLE; Schema: terrain; Owner: living_landz_srv
--

CREATE TABLE terrain.ocean_data (
    name character varying(32) NOT NULL,
    width integer NOT NULL,
    height integer NOT NULL,
    max_distance real NOT NULL,
    sdf_data bytea NOT NULL,
    heightmap_data bytea NOT NULL,
    generated_at bigint NOT NULL
);


ALTER TABLE terrain.ocean_data OWNER TO living_landz_srv;

--
-- Name: road_categories; Type: TABLE; Schema: terrain; Owner: living_landz_srv
--

CREATE TABLE terrain.road_categories (
    id smallint NOT NULL,
    name character varying NOT NULL
);


ALTER TABLE terrain.road_categories OWNER TO living_landz_srv;

--
-- Name: road_chunk_cache; Type: TABLE; Schema: terrain; Owner: living_landz_srv
--

CREATE TABLE terrain.road_chunk_cache (
    chunk_x integer NOT NULL,
    chunk_y integer NOT NULL,
    sdf_data bytea NOT NULL,
    sdf_width integer NOT NULL,
    sdf_height integer NOT NULL,
    segments_hash bytea NOT NULL,
    generated_at bigint NOT NULL
);


ALTER TABLE terrain.road_chunk_cache OWNER TO living_landz_srv;

--
-- Name: road_chunk_visibility; Type: TABLE; Schema: terrain; Owner: living_landz_srv
--

CREATE TABLE terrain.road_chunk_visibility (
    segment_id bigint NOT NULL,
    chunk_x integer NOT NULL,
    chunk_y integer NOT NULL,
    is_endpoint boolean DEFAULT false NOT NULL
);


ALTER TABLE terrain.road_chunk_visibility OWNER TO living_landz_srv;

--
-- Name: road_segments; Type: TABLE; Schema: terrain; Owner: living_landz_srv
--

CREATE TABLE terrain.road_segments (
    id bigint NOT NULL,
    start_q integer NOT NULL,
    start_r integer NOT NULL,
    end_q integer NOT NULL,
    end_r integer NOT NULL,
    points bytea NOT NULL,
    importance smallint NOT NULL,
    chunk_x integer NOT NULL,
    chunk_y integer NOT NULL,
    created_at bigint NOT NULL,
    updated_at bigint NOT NULL,
    cell_path bytea,
    road_type_id integer DEFAULT 1 NOT NULL,
    CONSTRAINT road_segments_importance_check CHECK (((importance >= 0) AND (importance <= 3)))
);


ALTER TABLE terrain.road_segments OWNER TO living_landz_srv;

--
-- Name: road_segments_id_seq; Type: SEQUENCE; Schema: terrain; Owner: living_landz_srv
--

CREATE SEQUENCE terrain.road_segments_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE terrain.road_segments_id_seq OWNER TO living_landz_srv;

--
-- Name: road_segments_id_seq; Type: SEQUENCE OWNED BY; Schema: terrain; Owner: living_landz_srv
--

ALTER SEQUENCE terrain.road_segments_id_seq OWNED BY terrain.road_segments.id;


--
-- Name: road_types; Type: TABLE; Schema: terrain; Owner: living_landz_srv
--

CREATE TABLE terrain.road_types (
    id integer NOT NULL,
    category_id smallint NOT NULL,
    variant character varying NOT NULL,
    archived boolean DEFAULT false
);


ALTER TABLE terrain.road_types OWNER TO living_landz_srv;

--
-- Name: road_types_id_seq; Type: SEQUENCE; Schema: terrain; Owner: living_landz_srv
--

CREATE SEQUENCE terrain.road_types_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE terrain.road_types_id_seq OWNER TO living_landz_srv;

--
-- Name: road_types_id_seq; Type: SEQUENCE OWNED BY; Schema: terrain; Owner: living_landz_srv
--

ALTER SEQUENCE terrain.road_types_id_seq OWNED BY terrain.road_types.id;


--
-- Name: terrain_biomes; Type: TABLE; Schema: terrain; Owner: living_landz_srv
--

CREATE TABLE terrain.terrain_biomes (
    name character varying(32) NOT NULL,
    chunk_x integer NOT NULL,
    chunk_y integer NOT NULL,
    biome_id smallint NOT NULL,
    data bytea NOT NULL,
    generated_at bigint NOT NULL
);


ALTER TABLE terrain.terrain_biomes OWNER TO living_landz_srv;

--
-- Name: terrains; Type: TABLE; Schema: terrain; Owner: living_landz_srv
--

CREATE TABLE terrain.terrains (
    name character varying(32) NOT NULL,
    chunk_x integer NOT NULL,
    chunk_y integer NOT NULL,
    data bytea NOT NULL,
    generated_at bigint NOT NULL
);


ALTER TABLE terrain.terrains OWNER TO living_landz_srv;

--
-- Name: voronoi_zone_cells; Type: TABLE; Schema: terrain; Owner: living_landz_srv
--

CREATE TABLE terrain.voronoi_zone_cells (
    zone_id bigint NOT NULL,
    cell_q integer NOT NULL,
    cell_r integer NOT NULL
);


ALTER TABLE terrain.voronoi_zone_cells OWNER TO living_landz_srv;

--
-- Name: voronoi_zones; Type: TABLE; Schema: terrain; Owner: living_landz_srv
--

CREATE TABLE terrain.voronoi_zones (
    id bigint NOT NULL,
    seed_cell_q integer NOT NULL,
    seed_cell_r integer NOT NULL,
    biome_type integer NOT NULL,
    cell_count integer DEFAULT 0,
    area_m2 real DEFAULT 0,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE terrain.voronoi_zones OWNER TO living_landz_srv;

--
-- Name: voronoi_zones_id_seq; Type: SEQUENCE; Schema: terrain; Owner: living_landz_srv
--

CREATE SEQUENCE terrain.voronoi_zones_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE terrain.voronoi_zones_id_seq OWNER TO living_landz_srv;

--
-- Name: voronoi_zones_id_seq; Type: SEQUENCE OWNED BY; Schema: terrain; Owner: living_landz_srv
--

ALTER SEQUENCE terrain.voronoi_zones_id_seq OWNED BY terrain.voronoi_zones.id;


--
-- Name: profession_skill_bonuses; Type: TABLE; Schema: units; Owner: living_landz_srv
--

CREATE TABLE units.profession_skill_bonuses (
    profession_id smallint NOT NULL,
    skill_id smallint NOT NULL,
    bonus_percentage integer NOT NULL
);


ALTER TABLE units.profession_skill_bonuses OWNER TO living_landz_srv;

--
-- Name: professions; Type: TABLE; Schema: units; Owner: living_landz_srv
--

CREATE TABLE units.professions (
    id smallint NOT NULL,
    name character varying NOT NULL,
    description text,
    base_inventory_capacity_bonus integer DEFAULT 0,
    archived boolean DEFAULT false,
    slug character varying(64)
);


ALTER TABLE units.professions OWNER TO living_landz_srv;

--
-- Name: skills; Type: TABLE; Schema: units; Owner: living_landz_srv
--

CREATE TABLE units.skills (
    id smallint NOT NULL,
    name character varying NOT NULL,
    description text,
    primary_stat character varying NOT NULL,
    archived boolean DEFAULT false,
    slug character varying(64)
);


ALTER TABLE units.skills OWNER TO living_landz_srv;

--
-- Name: unit_automated_actions; Type: TABLE; Schema: units; Owner: living_landz_srv
--

CREATE TABLE units.unit_automated_actions (
    id bigint NOT NULL,
    unit_id bigint NOT NULL,
    action_type character varying(100) NOT NULL,
    is_enabled boolean DEFAULT true,
    parameters jsonb,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);


ALTER TABLE units.unit_automated_actions OWNER TO living_landz_srv;

--
-- Name: TABLE unit_automated_actions; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON TABLE units.unit_automated_actions IS 'Automated actions configured for each unit';


--
-- Name: COLUMN unit_automated_actions.parameters; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON COLUMN units.unit_automated_actions.parameters IS 'JSON configuration for the action';


--
-- Name: unit_automated_actions_id_seq; Type: SEQUENCE; Schema: units; Owner: living_landz_srv
--

CREATE SEQUENCE units.unit_automated_actions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE units.unit_automated_actions_id_seq OWNER TO living_landz_srv;

--
-- Name: unit_automated_actions_id_seq; Type: SEQUENCE OWNED BY; Schema: units; Owner: living_landz_srv
--

ALTER SEQUENCE units.unit_automated_actions_id_seq OWNED BY units.unit_automated_actions.id;


--
-- Name: unit_base_stats; Type: TABLE; Schema: units; Owner: living_landz_srv
--

CREATE TABLE units.unit_base_stats (
    unit_id bigint NOT NULL,
    strength integer DEFAULT 10 NOT NULL,
    agility integer DEFAULT 10 NOT NULL,
    constitution integer DEFAULT 10 NOT NULL,
    intelligence integer DEFAULT 10 NOT NULL,
    wisdom integer DEFAULT 10 NOT NULL,
    charisma integer DEFAULT 10 NOT NULL,
    updated_at timestamp with time zone DEFAULT now()
);


ALTER TABLE units.unit_base_stats OWNER TO living_landz_srv;

--
-- Name: TABLE unit_base_stats; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON TABLE units.unit_base_stats IS 'Primary stats for units - stored in DB';


--
-- Name: unit_consumption_demands; Type: TABLE; Schema: units; Owner: living_landz_srv
--

CREATE TABLE units.unit_consumption_demands (
    unit_id bigint NOT NULL,
    item_id integer NOT NULL,
    quantity_per_day numeric(10,3) NOT NULL,
    priority integer DEFAULT 5 NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    CONSTRAINT chk_priority CHECK (((priority >= 1) AND (priority <= 10))),
    CONSTRAINT chk_quantity_per_day CHECK ((quantity_per_day > (0)::numeric))
);


ALTER TABLE units.unit_consumption_demands OWNER TO living_landz_srv;

--
-- Name: TABLE unit_consumption_demands; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON TABLE units.unit_consumption_demands IS 'Daily consumption needs for each unit';


--
-- Name: COLUMN unit_consumption_demands.priority; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON COLUMN units.unit_consumption_demands.priority IS 'Priority 1-10, where 1 is highest priority';


--
-- Name: unit_derived_stats; Type: TABLE; Schema: units; Owner: living_landz_srv
--

CREATE TABLE units.unit_derived_stats (
    unit_id bigint NOT NULL,
    max_hp integer DEFAULT 100 NOT NULL,
    current_hp integer DEFAULT 100 NOT NULL,
    happiness integer DEFAULT 50 NOT NULL,
    mental_health integer DEFAULT 100 NOT NULL,
    base_inventory_capacity_kg integer DEFAULT 50 NOT NULL,
    current_weight_kg numeric(10,3) DEFAULT 0 NOT NULL,
    updated_at timestamp with time zone DEFAULT now(),
    CONSTRAINT chk_current_hp CHECK (((current_hp >= 0) AND (current_hp <= max_hp))),
    CONSTRAINT chk_happiness CHECK (((happiness >= 0) AND (happiness <= 100))),
    CONSTRAINT chk_mental_health CHECK (((mental_health >= 0) AND (mental_health <= 100)))
);


ALTER TABLE units.unit_derived_stats OWNER TO living_landz_srv;

--
-- Name: TABLE unit_derived_stats; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON TABLE units.unit_derived_stats IS 'Derived stats - some can be calculated but stored for performance';


--
-- Name: COLUMN unit_derived_stats.base_inventory_capacity_kg; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON COLUMN units.unit_derived_stats.base_inventory_capacity_kg IS 'Base capacity, actual capacity includes equipment bonuses';


--
-- Name: unit_equipment; Type: TABLE; Schema: units; Owner: living_landz_srv
--

CREATE TABLE units.unit_equipment (
    unit_id bigint NOT NULL,
    equipment_slot_id smallint NOT NULL,
    item_id integer NOT NULL,
    equipped_at timestamp with time zone DEFAULT now()
);


ALTER TABLE units.unit_equipment OWNER TO living_landz_srv;

--
-- Name: TABLE unit_equipment; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON TABLE units.unit_equipment IS 'Currently equipped items for each unit';


--
-- Name: unit_inventory; Type: TABLE; Schema: units; Owner: living_landz_srv
--

CREATE TABLE units.unit_inventory (
    id bigint NOT NULL,
    unit_id bigint NOT NULL,
    item_id integer NOT NULL,
    quantity integer DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    CONSTRAINT chk_quantity CHECK ((quantity > 0))
);


ALTER TABLE units.unit_inventory OWNER TO living_landz_srv;

--
-- Name: TABLE unit_inventory; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON TABLE units.unit_inventory IS 'Items owned by each unit';


--
-- Name: unit_inventory_id_seq; Type: SEQUENCE; Schema: units; Owner: living_landz_srv
--

CREATE SEQUENCE units.unit_inventory_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE units.unit_inventory_id_seq OWNER TO living_landz_srv;

--
-- Name: unit_inventory_id_seq; Type: SEQUENCE OWNED BY; Schema: units; Owner: living_landz_srv
--

ALTER SEQUENCE units.unit_inventory_id_seq OWNED BY units.unit_inventory.id;


--
-- Name: unit_skills; Type: TABLE; Schema: units; Owner: living_landz_srv
--

CREATE TABLE units.unit_skills (
    unit_id bigint NOT NULL,
    skill_id smallint NOT NULL,
    xp bigint DEFAULT 0 NOT NULL,
    level integer DEFAULT 1 NOT NULL,
    updated_at timestamp with time zone DEFAULT now()
);


ALTER TABLE units.unit_skills OWNER TO living_landz_srv;

--
-- Name: TABLE unit_skills; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON TABLE units.unit_skills IS 'Skills and experience for each unit';


--
-- Name: units; Type: TABLE; Schema: units; Owner: living_landz_srv
--

CREATE TABLE units.units (
    id bigint NOT NULL,
    player_id bigint,
    first_name character varying(100) NOT NULL,
    last_name character varying(100) NOT NULL,
    level integer DEFAULT 1 NOT NULL,
    avatar_url character varying(500),
    current_cell_q integer NOT NULL,
    current_cell_r integer NOT NULL,
    current_chunk_x integer NOT NULL,
    current_chunk_y integer NOT NULL,
    profession_id smallint DEFAULT 0 NOT NULL,
    money bigint DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    slot_type character varying(20),
    slot_index integer,
    gender character varying(10) DEFAULT 'male'::character varying NOT NULL,
    portrait_variant_id character varying(10),
    is_lord boolean DEFAULT false NOT NULL,
    portrait_layers text,
    CONSTRAINT check_gender CHECK (((gender)::text = ANY ((ARRAY['male'::character varying, 'female'::character varying])::text[])))
);


ALTER TABLE units.units OWNER TO living_landz_srv;

--
-- Name: TABLE units; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON TABLE units.units IS 'Main table for all units (players and NPCs)';


--
-- Name: COLUMN units.player_id; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON COLUMN units.units.player_id IS 'NULL for NPCs, player ID for player characters';


--
-- Name: COLUMN units.slot_type; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON COLUMN units.units.slot_type IS 'Type of slot (interior/exterior) where unit is positioned within the cell';


--
-- Name: COLUMN units.slot_index; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON COLUMN units.units.slot_index IS 'Index of the slot within the slot type grid (0-based)';


--
-- Name: COLUMN units.is_lord; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON COLUMN units.units.is_lord IS 'True if this unit is the player''s Lord/Lady (main avatar on the map)';


--
-- Name: COLUMN units.portrait_layers; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON COLUMN units.units.portrait_layers IS 'Portrait layer indices as comma-separated values: bust,face,clothes,hair';


--
-- Name: units_id_seq; Type: SEQUENCE; Schema: units; Owner: living_landz_srv
--

CREATE SEQUENCE units.units_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE units.units_id_seq OWNER TO living_landz_srv;

--
-- Name: units_id_seq; Type: SEQUENCE OWNED BY; Schema: units; Owner: living_landz_srv
--

ALTER SEQUENCE units.units_id_seq OWNED BY units.units.id;


--
-- Name: scheduled_actions id; Type: DEFAULT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.scheduled_actions ALTER COLUMN id SET DEFAULT nextval('actions.scheduled_actions_id_seq'::regclass);


--
-- Name: send_message_receivers id; Type: DEFAULT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.send_message_receivers ALTER COLUMN id SET DEFAULT nextval('actions.send_message_receivers_id_seq'::regclass);


--
-- Name: building_types id; Type: DEFAULT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.building_types ALTER COLUMN id SET DEFAULT nextval('buildings.building_types_id_seq'::regclass);


--
-- Name: buildings_base id; Type: DEFAULT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.buildings_base ALTER COLUMN id SET DEFAULT nextval('buildings.buildings_base_id_seq'::regclass);


--
-- Name: characters id; Type: DEFAULT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.characters ALTER COLUMN id SET DEFAULT nextval('game.characters_id_seq'::regclass);


--
-- Name: coats_of_arms id; Type: DEFAULT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.coats_of_arms ALTER COLUMN id SET DEFAULT nextval('game.coats_of_arms_id_seq'::regclass);


--
-- Name: players id; Type: DEFAULT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.players ALTER COLUMN id SET DEFAULT nextval('game.players_id_seq'::regclass);


--
-- Name: buildings id; Type: DEFAULT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.buildings ALTER COLUMN id SET DEFAULT nextval('organizations.buildings_id_seq'::regclass);


--
-- Name: diplomatic_relations id; Type: DEFAULT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.diplomatic_relations ALTER COLUMN id SET DEFAULT nextval('organizations.diplomatic_relations_id_seq'::regclass);


--
-- Name: members id; Type: DEFAULT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.members ALTER COLUMN id SET DEFAULT nextval('organizations.members_id_seq'::regclass);


--
-- Name: officers id; Type: DEFAULT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.officers ALTER COLUMN id SET DEFAULT nextval('organizations.officers_id_seq'::regclass);


--
-- Name: organizations id; Type: DEFAULT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.organizations ALTER COLUMN id SET DEFAULT nextval('organizations.organizations_id_seq'::regclass);


--
-- Name: territory_contours id; Type: DEFAULT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.territory_contours ALTER COLUMN id SET DEFAULT nextval('organizations.territory_contours_id_seq'::regclass);


--
-- Name: treasury_items id; Type: DEFAULT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.treasury_items ALTER COLUMN id SET DEFAULT nextval('organizations.treasury_items_id_seq'::regclass);


--
-- Name: harvest_yields id; Type: DEFAULT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.harvest_yields ALTER COLUMN id SET DEFAULT nextval('resources.harvest_yields_id_seq'::regclass);


--
-- Name: item_instances id; Type: DEFAULT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.item_instances ALTER COLUMN id SET DEFAULT nextval('resources.item_instances_id_seq'::regclass);


--
-- Name: items id; Type: DEFAULT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.items ALTER COLUMN id SET DEFAULT nextval('resources.items_id_seq'::regclass);


--
-- Name: recipes id; Type: DEFAULT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.recipes ALTER COLUMN id SET DEFAULT nextval('resources.recipes_id_seq'::regclass);


--
-- Name: resource_types id; Type: DEFAULT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.resource_types ALTER COLUMN id SET DEFAULT nextval('resources.resource_types_id_seq'::regclass);


--
-- Name: road_segments id; Type: DEFAULT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.road_segments ALTER COLUMN id SET DEFAULT nextval('terrain.road_segments_id_seq'::regclass);


--
-- Name: road_types id; Type: DEFAULT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.road_types ALTER COLUMN id SET DEFAULT nextval('terrain.road_types_id_seq'::regclass);


--
-- Name: voronoi_zones id; Type: DEFAULT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.voronoi_zones ALTER COLUMN id SET DEFAULT nextval('terrain.voronoi_zones_id_seq'::regclass);


--
-- Name: unit_automated_actions id; Type: DEFAULT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_automated_actions ALTER COLUMN id SET DEFAULT nextval('units.unit_automated_actions_id_seq'::regclass);


--
-- Name: unit_inventory id; Type: DEFAULT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_inventory ALTER COLUMN id SET DEFAULT nextval('units.unit_inventory_id_seq'::regclass);


--
-- Name: units id; Type: DEFAULT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.units ALTER COLUMN id SET DEFAULT nextval('units.units_id_seq'::regclass);


--
-- Name: action_specific_types action_specific_types_name_key; Type: CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.action_specific_types
    ADD CONSTRAINT action_specific_types_name_key UNIQUE (name);


--
-- Name: action_specific_types action_specific_types_pkey; Type: CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.action_specific_types
    ADD CONSTRAINT action_specific_types_pkey PRIMARY KEY (id);


--
-- Name: action_statuses action_statuses_name_key; Type: CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.action_statuses
    ADD CONSTRAINT action_statuses_name_key UNIQUE (name);


--
-- Name: action_statuses action_statuses_pkey; Type: CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.action_statuses
    ADD CONSTRAINT action_statuses_pkey PRIMARY KEY (id);


--
-- Name: action_types action_types_name_key; Type: CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.action_types
    ADD CONSTRAINT action_types_name_key UNIQUE (name);


--
-- Name: action_types action_types_pkey; Type: CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.action_types
    ADD CONSTRAINT action_types_pkey PRIMARY KEY (id);


--
-- Name: build_building_actions build_building_actions_pkey; Type: CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.build_building_actions
    ADD CONSTRAINT build_building_actions_pkey PRIMARY KEY (action_id);


--
-- Name: build_road_actions build_road_actions_pkey; Type: CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.build_road_actions
    ADD CONSTRAINT build_road_actions_pkey PRIMARY KEY (action_id);


--
-- Name: craft_resource_actions craft_resource_actions_pkey; Type: CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.craft_resource_actions
    ADD CONSTRAINT craft_resource_actions_pkey PRIMARY KEY (action_id);


--
-- Name: harvest_resource_actions harvest_resource_actions_pkey; Type: CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.harvest_resource_actions
    ADD CONSTRAINT harvest_resource_actions_pkey PRIMARY KEY (action_id);


--
-- Name: move_unit_actions move_unit_actions_pkey; Type: CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.move_unit_actions
    ADD CONSTRAINT move_unit_actions_pkey PRIMARY KEY (action_id);


--
-- Name: scheduled_actions scheduled_actions_pkey; Type: CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.scheduled_actions
    ADD CONSTRAINT scheduled_actions_pkey PRIMARY KEY (id);


--
-- Name: send_message_actions send_message_actions_pkey; Type: CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.send_message_actions
    ADD CONSTRAINT send_message_actions_pkey PRIMARY KEY (action_id);


--
-- Name: send_message_receivers send_message_receivers_pkey; Type: CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.send_message_receivers
    ADD CONSTRAINT send_message_receivers_pkey PRIMARY KEY (id);


--
-- Name: train_unit_actions train_unit_actions_pkey; Type: CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.train_unit_actions
    ADD CONSTRAINT train_unit_actions_pkey PRIMARY KEY (action_id);


--
-- Name: agriculture agriculture_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.agriculture
    ADD CONSTRAINT agriculture_pkey PRIMARY KEY (building_id);


--
-- Name: agriculture_types agriculture_types_name_key; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.agriculture_types
    ADD CONSTRAINT agriculture_types_name_key UNIQUE (name);


--
-- Name: agriculture_types agriculture_types_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.agriculture_types
    ADD CONSTRAINT agriculture_types_pkey PRIMARY KEY (id);


--
-- Name: animal_breeding animal_breeding_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.animal_breeding
    ADD CONSTRAINT animal_breeding_pkey PRIMARY KEY (building_id);


--
-- Name: animal_breeding_types animal_breeding_types_name_key; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.animal_breeding_types
    ADD CONSTRAINT animal_breeding_types_name_key UNIQUE (name);


--
-- Name: animal_breeding_types animal_breeding_types_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.animal_breeding_types
    ADD CONSTRAINT animal_breeding_types_pkey PRIMARY KEY (id);


--
-- Name: building_categories building_categories_name_key; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.building_categories
    ADD CONSTRAINT building_categories_name_key UNIQUE (name);


--
-- Name: building_categories building_categories_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.building_categories
    ADD CONSTRAINT building_categories_pkey PRIMARY KEY (id);


--
-- Name: building_categories building_categories_slug_key; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.building_categories
    ADD CONSTRAINT building_categories_slug_key UNIQUE (slug);


--
-- Name: building_specific_types building_specific_types_name_key; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.building_specific_types
    ADD CONSTRAINT building_specific_types_name_key UNIQUE (name);


--
-- Name: building_specific_types building_specific_types_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.building_specific_types
    ADD CONSTRAINT building_specific_types_pkey PRIMARY KEY (id);


--
-- Name: building_specific_types building_specific_types_slug_key; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.building_specific_types
    ADD CONSTRAINT building_specific_types_slug_key UNIQUE (slug);


--
-- Name: building_types building_types_name_category_id_specific_type_id_key; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.building_types
    ADD CONSTRAINT building_types_name_category_id_specific_type_id_key UNIQUE (name, category_id, specific_type_id);


--
-- Name: building_types building_types_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.building_types
    ADD CONSTRAINT building_types_pkey PRIMARY KEY (id);


--
-- Name: building_types building_types_slug_key; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.building_types
    ADD CONSTRAINT building_types_slug_key UNIQUE (slug);


--
-- Name: buildings_base buildings_base_cell_q_cell_r_chunk_x_chunk_y_key; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.buildings_base
    ADD CONSTRAINT buildings_base_cell_q_cell_r_chunk_x_chunk_y_key UNIQUE (cell_q, cell_r, chunk_x, chunk_y);


--
-- Name: buildings_base buildings_base_cell_q_cell_r_key; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.buildings_base
    ADD CONSTRAINT buildings_base_cell_q_cell_r_key UNIQUE (cell_q, cell_r);


--
-- Name: buildings_base buildings_base_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.buildings_base
    ADD CONSTRAINT buildings_base_pkey PRIMARY KEY (id);


--
-- Name: commerce commerce_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.commerce
    ADD CONSTRAINT commerce_pkey PRIMARY KEY (building_id);


--
-- Name: commerce_types commerce_types_name_key; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.commerce_types
    ADD CONSTRAINT commerce_types_name_key UNIQUE (name);


--
-- Name: commerce_types commerce_types_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.commerce_types
    ADD CONSTRAINT commerce_types_pkey PRIMARY KEY (id);


--
-- Name: construction_costs construction_costs_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.construction_costs
    ADD CONSTRAINT construction_costs_pkey PRIMARY KEY (building_type_id, item_id);


--
-- Name: cult cult_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.cult
    ADD CONSTRAINT cult_pkey PRIMARY KEY (building_id);


--
-- Name: cult_types cult_types_name_key; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.cult_types
    ADD CONSTRAINT cult_types_name_key UNIQUE (name);


--
-- Name: cult_types cult_types_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.cult_types
    ADD CONSTRAINT cult_types_pkey PRIMARY KEY (id);


--
-- Name: entertainment entertainment_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.entertainment
    ADD CONSTRAINT entertainment_pkey PRIMARY KEY (building_id);


--
-- Name: entertainment_types entertainment_types_name_key; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.entertainment_types
    ADD CONSTRAINT entertainment_types_name_key UNIQUE (name);


--
-- Name: entertainment_types entertainment_types_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.entertainment_types
    ADD CONSTRAINT entertainment_types_pkey PRIMARY KEY (id);


--
-- Name: manufacturing_workshop_types manufacturing_workshop_types_name_key; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.manufacturing_workshop_types
    ADD CONSTRAINT manufacturing_workshop_types_name_key UNIQUE (name);


--
-- Name: manufacturing_workshop_types manufacturing_workshop_types_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.manufacturing_workshop_types
    ADD CONSTRAINT manufacturing_workshop_types_pkey PRIMARY KEY (id);


--
-- Name: manufacturing_workshops manufacturing_workshops_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.manufacturing_workshops
    ADD CONSTRAINT manufacturing_workshops_pkey PRIMARY KEY (building_id);


--
-- Name: tree_types tree_types_name_key; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.tree_types
    ADD CONSTRAINT tree_types_name_key UNIQUE (name);


--
-- Name: tree_types tree_types_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.tree_types
    ADD CONSTRAINT tree_types_pkey PRIMARY KEY (id);


--
-- Name: trees trees_pkey; Type: CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.trees
    ADD CONSTRAINT trees_pkey PRIMARY KEY (building_id);


--
-- Name: characters characters_pkey; Type: CONSTRAINT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.characters
    ADD CONSTRAINT characters_pkey PRIMARY KEY (id);


--
-- Name: characters characters_player_id_first_name_family_name_key; Type: CONSTRAINT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.characters
    ADD CONSTRAINT characters_player_id_first_name_family_name_key UNIQUE (player_id, first_name, family_name);


--
-- Name: coats_of_arms coats_of_arms_pkey; Type: CONSTRAINT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.coats_of_arms
    ADD CONSTRAINT coats_of_arms_pkey PRIMARY KEY (id);


--
-- Name: languages languages_code_key; Type: CONSTRAINT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.languages
    ADD CONSTRAINT languages_code_key UNIQUE (code);


--
-- Name: languages languages_name_key; Type: CONSTRAINT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.languages
    ADD CONSTRAINT languages_name_key UNIQUE (name);


--
-- Name: languages languages_pkey; Type: CONSTRAINT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.languages
    ADD CONSTRAINT languages_pkey PRIMARY KEY (id);


--
-- Name: players players_email_key; Type: CONSTRAINT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.players
    ADD CONSTRAINT players_email_key UNIQUE (email);


--
-- Name: players players_pkey; Type: CONSTRAINT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.players
    ADD CONSTRAINT players_pkey PRIMARY KEY (id);


--
-- Name: translations translations_pkey; Type: CONSTRAINT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.translations
    ADD CONSTRAINT translations_pkey PRIMARY KEY (entity_type, entity_id, language_id, field);


--
-- Name: buildings buildings_building_id_key; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.buildings
    ADD CONSTRAINT buildings_building_id_key UNIQUE (building_id);


--
-- Name: buildings buildings_pkey; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.buildings
    ADD CONSTRAINT buildings_pkey PRIMARY KEY (id);


--
-- Name: diplomatic_relations diplomatic_relations_organization_id_target_organization_id_key; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.diplomatic_relations
    ADD CONSTRAINT diplomatic_relations_organization_id_target_organization_id_key UNIQUE (organization_id, target_organization_id);


--
-- Name: diplomatic_relations diplomatic_relations_pkey; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.diplomatic_relations
    ADD CONSTRAINT diplomatic_relations_pkey PRIMARY KEY (id);


--
-- Name: members members_organization_id_unit_id_key; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.members
    ADD CONSTRAINT members_organization_id_unit_id_key UNIQUE (organization_id, unit_id);


--
-- Name: members members_pkey; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.members
    ADD CONSTRAINT members_pkey PRIMARY KEY (id);


--
-- Name: officers officers_organization_id_unit_id_role_type_id_key; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.officers
    ADD CONSTRAINT officers_organization_id_unit_id_role_type_id_key UNIQUE (organization_id, unit_id, role_type_id);


--
-- Name: officers officers_pkey; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.officers
    ADD CONSTRAINT officers_pkey PRIMARY KEY (id);


--
-- Name: organization_role_compatibility organization_role_compatibility_pkey; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.organization_role_compatibility
    ADD CONSTRAINT organization_role_compatibility_pkey PRIMARY KEY (organization_type_id, role_type_id);


--
-- Name: organization_types organization_types_name_key; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.organization_types
    ADD CONSTRAINT organization_types_name_key UNIQUE (name);


--
-- Name: organization_types organization_types_pkey; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.organization_types
    ADD CONSTRAINT organization_types_pkey PRIMARY KEY (id);


--
-- Name: organizations organizations_pkey; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.organizations
    ADD CONSTRAINT organizations_pkey PRIMARY KEY (id);


--
-- Name: role_types role_types_name_key; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.role_types
    ADD CONSTRAINT role_types_name_key UNIQUE (name);


--
-- Name: role_types role_types_pkey; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.role_types
    ADD CONSTRAINT role_types_pkey PRIMARY KEY (id);


--
-- Name: territory_cells territory_cells_cell_q_cell_r_key; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.territory_cells
    ADD CONSTRAINT territory_cells_cell_q_cell_r_key UNIQUE (cell_q, cell_r);


--
-- Name: territory_cells territory_cells_pkey; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.territory_cells
    ADD CONSTRAINT territory_cells_pkey PRIMARY KEY (organization_id, cell_q, cell_r);


--
-- Name: territory_contours territory_contours_organization_id_chunk_x_chunk_y_key; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.territory_contours
    ADD CONSTRAINT territory_contours_organization_id_chunk_x_chunk_y_key UNIQUE (organization_id, chunk_x, chunk_y);


--
-- Name: territory_contours territory_contours_pkey; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.territory_contours
    ADD CONSTRAINT territory_contours_pkey PRIMARY KEY (id);


--
-- Name: treasury_items treasury_items_organization_id_item_instance_id_key; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.treasury_items
    ADD CONSTRAINT treasury_items_organization_id_item_instance_id_key UNIQUE (organization_id, item_instance_id);


--
-- Name: treasury_items treasury_items_pkey; Type: CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.treasury_items
    ADD CONSTRAINT treasury_items_pkey PRIMARY KEY (id);


--
-- Name: _sqlx_migrations _sqlx_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: living_landz_srv
--

ALTER TABLE ONLY public._sqlx_migrations
    ADD CONSTRAINT _sqlx_migrations_pkey PRIMARY KEY (version);


--
-- Name: equipment_slots equipment_slots_name_key; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.equipment_slots
    ADD CONSTRAINT equipment_slots_name_key UNIQUE (name);


--
-- Name: equipment_slots equipment_slots_pkey; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.equipment_slots
    ADD CONSTRAINT equipment_slots_pkey PRIMARY KEY (id);


--
-- Name: equipment_slots equipment_slots_slug_key; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.equipment_slots
    ADD CONSTRAINT equipment_slots_slug_key UNIQUE (slug);


--
-- Name: harvest_yields harvest_yields_pkey; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.harvest_yields
    ADD CONSTRAINT harvest_yields_pkey PRIMARY KEY (id);


--
-- Name: item_instances item_instances_pkey; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.item_instances
    ADD CONSTRAINT item_instances_pkey PRIMARY KEY (id);


--
-- Name: item_stat_modifiers item_stat_modifiers_pkey; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.item_stat_modifiers
    ADD CONSTRAINT item_stat_modifiers_pkey PRIMARY KEY (item_id, stat_name);


--
-- Name: item_types item_types_name_key; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.item_types
    ADD CONSTRAINT item_types_name_key UNIQUE (name);


--
-- Name: item_types item_types_pkey; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.item_types
    ADD CONSTRAINT item_types_pkey PRIMARY KEY (id);


--
-- Name: item_types item_types_slug_key; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.item_types
    ADD CONSTRAINT item_types_slug_key UNIQUE (slug);


--
-- Name: items items_name_key; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.items
    ADD CONSTRAINT items_name_key UNIQUE (name);


--
-- Name: items items_pkey; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.items
    ADD CONSTRAINT items_pkey PRIMARY KEY (id);


--
-- Name: items items_slug_key; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.items
    ADD CONSTRAINT items_slug_key UNIQUE (slug);


--
-- Name: recipe_ingredients recipe_ingredients_pkey; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.recipe_ingredients
    ADD CONSTRAINT recipe_ingredients_pkey PRIMARY KEY (recipe_id, item_id);


--
-- Name: recipes recipes_name_key; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.recipes
    ADD CONSTRAINT recipes_name_key UNIQUE (name);


--
-- Name: recipes recipes_pkey; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.recipes
    ADD CONSTRAINT recipes_pkey PRIMARY KEY (id);


--
-- Name: recipes recipes_slug_key; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.recipes
    ADD CONSTRAINT recipes_slug_key UNIQUE (slug);


--
-- Name: resource_categories resource_categories_name_key; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.resource_categories
    ADD CONSTRAINT resource_categories_name_key UNIQUE (name);


--
-- Name: resource_categories resource_categories_pkey; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.resource_categories
    ADD CONSTRAINT resource_categories_pkey PRIMARY KEY (id);


--
-- Name: resource_categories resource_categories_slug_key; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.resource_categories
    ADD CONSTRAINT resource_categories_slug_key UNIQUE (slug);


--
-- Name: resource_specific_types resource_specific_types_name_key; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.resource_specific_types
    ADD CONSTRAINT resource_specific_types_name_key UNIQUE (name);


--
-- Name: resource_specific_types resource_specific_types_pkey; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.resource_specific_types
    ADD CONSTRAINT resource_specific_types_pkey PRIMARY KEY (id);


--
-- Name: resource_specific_types resource_specific_types_slug_key; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.resource_specific_types
    ADD CONSTRAINT resource_specific_types_slug_key UNIQUE (slug);


--
-- Name: resource_types resource_types_name_category_id_specific_type_id_key; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.resource_types
    ADD CONSTRAINT resource_types_name_category_id_specific_type_id_key UNIQUE (name, category_id, specific_type_id);


--
-- Name: resource_types resource_types_name_key; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.resource_types
    ADD CONSTRAINT resource_types_name_key UNIQUE (name);


--
-- Name: resource_types resource_types_pkey; Type: CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.resource_types
    ADD CONSTRAINT resource_types_pkey PRIMARY KEY (id);


--
-- Name: biome_types biome_types_name_key; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.biome_types
    ADD CONSTRAINT biome_types_name_key UNIQUE (name);


--
-- Name: biome_types biome_types_pkey; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.biome_types
    ADD CONSTRAINT biome_types_pkey PRIMARY KEY (id);


--
-- Name: cells cells_chunk_x_chunk_y_q_r_key; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.cells
    ADD CONSTRAINT cells_chunk_x_chunk_y_q_r_key UNIQUE (chunk_x, chunk_y, q, r);


--
-- Name: cells cells_pkey; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.cells
    ADD CONSTRAINT cells_pkey PRIMARY KEY (q, r);


--
-- Name: ocean_data ocean_data_pkey; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.ocean_data
    ADD CONSTRAINT ocean_data_pkey PRIMARY KEY (name);


--
-- Name: road_categories road_categories_name_key; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.road_categories
    ADD CONSTRAINT road_categories_name_key UNIQUE (name);


--
-- Name: road_categories road_categories_pkey; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.road_categories
    ADD CONSTRAINT road_categories_pkey PRIMARY KEY (id);


--
-- Name: road_chunk_cache road_chunk_cache_pkey; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.road_chunk_cache
    ADD CONSTRAINT road_chunk_cache_pkey PRIMARY KEY (chunk_x, chunk_y);


--
-- Name: road_chunk_visibility road_chunk_visibility_pkey; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.road_chunk_visibility
    ADD CONSTRAINT road_chunk_visibility_pkey PRIMARY KEY (segment_id, chunk_x, chunk_y);


--
-- Name: road_segments road_segments_pkey; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.road_segments
    ADD CONSTRAINT road_segments_pkey PRIMARY KEY (id);


--
-- Name: road_types road_types_category_id_variant_key; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.road_types
    ADD CONSTRAINT road_types_category_id_variant_key UNIQUE (category_id, variant);


--
-- Name: road_types road_types_pkey; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.road_types
    ADD CONSTRAINT road_types_pkey PRIMARY KEY (id);


--
-- Name: terrain_biomes terrain_biomes_pkey; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.terrain_biomes
    ADD CONSTRAINT terrain_biomes_pkey PRIMARY KEY (name, chunk_x, chunk_y, biome_id);


--
-- Name: terrains terrains_pkey; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.terrains
    ADD CONSTRAINT terrains_pkey PRIMARY KEY (name, chunk_x, chunk_y);


--
-- Name: road_segments unique_segment; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.road_segments
    ADD CONSTRAINT unique_segment UNIQUE (start_q, start_r, end_q, end_r);


--
-- Name: voronoi_zone_cells voronoi_zone_cells_pkey; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.voronoi_zone_cells
    ADD CONSTRAINT voronoi_zone_cells_pkey PRIMARY KEY (cell_q, cell_r);


--
-- Name: voronoi_zones voronoi_zones_pkey; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.voronoi_zones
    ADD CONSTRAINT voronoi_zones_pkey PRIMARY KEY (id);


--
-- Name: voronoi_zones voronoi_zones_seed_cell_q_seed_cell_r_key; Type: CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.voronoi_zones
    ADD CONSTRAINT voronoi_zones_seed_cell_q_seed_cell_r_key UNIQUE (seed_cell_q, seed_cell_r);


--
-- Name: profession_skill_bonuses profession_skill_bonuses_pkey; Type: CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.profession_skill_bonuses
    ADD CONSTRAINT profession_skill_bonuses_pkey PRIMARY KEY (profession_id, skill_id);


--
-- Name: professions professions_name_key; Type: CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.professions
    ADD CONSTRAINT professions_name_key UNIQUE (name);


--
-- Name: professions professions_pkey; Type: CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.professions
    ADD CONSTRAINT professions_pkey PRIMARY KEY (id);


--
-- Name: professions professions_slug_key; Type: CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.professions
    ADD CONSTRAINT professions_slug_key UNIQUE (slug);


--
-- Name: skills skills_name_key; Type: CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.skills
    ADD CONSTRAINT skills_name_key UNIQUE (name);


--
-- Name: skills skills_pkey; Type: CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.skills
    ADD CONSTRAINT skills_pkey PRIMARY KEY (id);


--
-- Name: skills skills_slug_key; Type: CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.skills
    ADD CONSTRAINT skills_slug_key UNIQUE (slug);


--
-- Name: unit_automated_actions unit_automated_actions_pkey; Type: CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_automated_actions
    ADD CONSTRAINT unit_automated_actions_pkey PRIMARY KEY (id);


--
-- Name: unit_base_stats unit_base_stats_pkey; Type: CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_base_stats
    ADD CONSTRAINT unit_base_stats_pkey PRIMARY KEY (unit_id);


--
-- Name: unit_consumption_demands unit_consumption_demands_pkey; Type: CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_consumption_demands
    ADD CONSTRAINT unit_consumption_demands_pkey PRIMARY KEY (unit_id, item_id);


--
-- Name: unit_derived_stats unit_derived_stats_pkey; Type: CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_derived_stats
    ADD CONSTRAINT unit_derived_stats_pkey PRIMARY KEY (unit_id);


--
-- Name: unit_equipment unit_equipment_pkey; Type: CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_equipment
    ADD CONSTRAINT unit_equipment_pkey PRIMARY KEY (unit_id, equipment_slot_id);


--
-- Name: unit_inventory unit_inventory_pkey; Type: CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_inventory
    ADD CONSTRAINT unit_inventory_pkey PRIMARY KEY (id);


--
-- Name: unit_inventory unit_inventory_unit_id_item_id_key; Type: CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_inventory
    ADD CONSTRAINT unit_inventory_unit_id_item_id_key UNIQUE (unit_id, item_id);


--
-- Name: unit_skills unit_skills_pkey; Type: CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_skills
    ADD CONSTRAINT unit_skills_pkey PRIMARY KEY (unit_id, skill_id);


--
-- Name: units units_pkey; Type: CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.units
    ADD CONSTRAINT units_pkey PRIMARY KEY (id);


--
-- Name: idx_building_type_id; Type: INDEX; Schema: actions; Owner: living_landz_srv
--

CREATE INDEX idx_building_type_id ON actions.build_building_actions USING btree (building_type_id);


--
-- Name: idx_chunk_commands; Type: INDEX; Schema: actions; Owner: living_landz_srv
--

CREATE INDEX idx_chunk_commands ON actions.scheduled_actions USING btree (chunk_x, chunk_y) WHERE (status_id = ANY (ARRAY[1, 2]));


--
-- Name: idx_completion_time; Type: INDEX; Schema: actions; Owner: living_landz_srv
--

CREATE INDEX idx_completion_time ON actions.scheduled_actions USING btree (completion_time);


--
-- Name: idx_message_receivers; Type: INDEX; Schema: actions; Owner: living_landz_srv
--

CREATE INDEX idx_message_receivers ON actions.send_message_receivers USING btree (action_id);


--
-- Name: idx_player_commands; Type: INDEX; Schema: actions; Owner: living_landz_srv
--

CREATE INDEX idx_player_commands ON actions.scheduled_actions USING btree (player_id) WHERE (status_id = ANY (ARRAY[1, 2]));


--
-- Name: idx_receiver_messages; Type: INDEX; Schema: actions; Owner: living_landz_srv
--

CREATE INDEX idx_receiver_messages ON actions.send_message_receivers USING btree (receiver_id);


--
-- Name: idx_resource_type_id; Type: INDEX; Schema: actions; Owner: living_landz_srv
--

CREATE INDEX idx_resource_type_id ON actions.harvest_resource_actions USING btree (resource_type_id);


--
-- Name: idx_train_unit_id; Type: INDEX; Schema: actions; Owner: living_landz_srv
--

CREATE INDEX idx_train_unit_id ON actions.train_unit_actions USING btree (unit_id);


--
-- Name: idx_building_agriculture; Type: INDEX; Schema: buildings; Owner: living_landz_srv
--

CREATE INDEX idx_building_agriculture ON buildings.agriculture USING btree (building_id);


--
-- Name: idx_building_animal_breeding; Type: INDEX; Schema: buildings; Owner: living_landz_srv
--

CREATE INDEX idx_building_animal_breeding ON buildings.animal_breeding USING btree (building_id);


--
-- Name: idx_building_commerce; Type: INDEX; Schema: buildings; Owner: living_landz_srv
--

CREATE INDEX idx_building_commerce ON buildings.commerce USING btree (building_id);


--
-- Name: idx_building_cult; Type: INDEX; Schema: buildings; Owner: living_landz_srv
--

CREATE INDEX idx_building_cult ON buildings.cult USING btree (building_id);


--
-- Name: idx_building_entertainment; Type: INDEX; Schema: buildings; Owner: living_landz_srv
--

CREATE INDEX idx_building_entertainment ON buildings.entertainment USING btree (building_id);


--
-- Name: idx_building_manufacturing_workshops; Type: INDEX; Schema: buildings; Owner: living_landz_srv
--

CREATE INDEX idx_building_manufacturing_workshops ON buildings.manufacturing_workshops USING btree (building_id);


--
-- Name: idx_building_trees; Type: INDEX; Schema: buildings; Owner: living_landz_srv
--

CREATE INDEX idx_building_trees ON buildings.trees USING btree (building_id);


--
-- Name: idx_building_type_id; Type: INDEX; Schema: buildings; Owner: living_landz_srv
--

CREATE INDEX idx_building_type_id ON buildings.buildings_base USING btree (building_type_id);


--
-- Name: idx_building_types_category; Type: INDEX; Schema: buildings; Owner: living_landz_srv
--

CREATE INDEX idx_building_types_category ON buildings.building_types USING btree (category_id);


--
-- Name: idx_building_types_specific; Type: INDEX; Schema: buildings; Owner: living_landz_srv
--

CREATE INDEX idx_building_types_specific ON buildings.building_types USING btree (specific_type_id);


--
-- Name: idx_buildings_chunk; Type: INDEX; Schema: buildings; Owner: living_landz_srv
--

CREATE INDEX idx_buildings_chunk ON buildings.buildings_base USING btree (chunk_x, chunk_y);


--
-- Name: idx_buildings_created; Type: INDEX; Schema: buildings; Owner: living_landz_srv
--

CREATE INDEX idx_buildings_created ON buildings.buildings_base USING btree (created_at);


--
-- Name: idx_buildings_is_built; Type: INDEX; Schema: buildings; Owner: living_landz_srv
--

CREATE INDEX idx_buildings_is_built ON buildings.buildings_base USING btree (is_built);


--
-- Name: idx_characters_coat_of_arms; Type: INDEX; Schema: game; Owner: living_landz_srv
--

CREATE INDEX idx_characters_coat_of_arms ON game.characters USING btree (coat_of_arms_id);


--
-- Name: idx_characters_created; Type: INDEX; Schema: game; Owner: living_landz_srv
--

CREATE INDEX idx_characters_created ON game.characters USING btree (created_at);


--
-- Name: idx_characters_name; Type: INDEX; Schema: game; Owner: living_landz_srv
--

CREATE INDEX idx_characters_name ON game.characters USING btree (first_name, family_name);


--
-- Name: idx_characters_player; Type: INDEX; Schema: game; Owner: living_landz_srv
--

CREATE INDEX idx_characters_player ON game.characters USING btree (player_id);


--
-- Name: idx_players_created; Type: INDEX; Schema: game; Owner: living_landz_srv
--

CREATE INDEX idx_players_created ON game.players USING btree (created_at);


--
-- Name: idx_players_email; Type: INDEX; Schema: game; Owner: living_landz_srv
--

CREATE INDEX idx_players_email ON game.players USING btree (email) WHERE (email IS NOT NULL);


--
-- Name: idx_players_family_name; Type: INDEX; Schema: game; Owner: living_landz_srv
--

CREATE INDEX idx_players_family_name ON game.players USING btree (family_name);


--
-- Name: idx_players_family_name_unique; Type: INDEX; Schema: game; Owner: living_landz_srv
--

CREATE UNIQUE INDEX idx_players_family_name_unique ON game.players USING btree (family_name);


--
-- Name: idx_translations_entity; Type: INDEX; Schema: game; Owner: living_landz_srv
--

CREATE INDEX idx_translations_entity ON game.translations USING btree (entity_type, entity_id);


--
-- Name: idx_diplomatic_org; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_diplomatic_org ON organizations.diplomatic_relations USING btree (organization_id);


--
-- Name: idx_diplomatic_target; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_diplomatic_target ON organizations.diplomatic_relations USING btree (target_organization_id);


--
-- Name: idx_diplomatic_type; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_diplomatic_type ON organizations.diplomatic_relations USING btree (relation_type);


--
-- Name: idx_members_org; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_members_org ON organizations.members USING btree (organization_id);


--
-- Name: idx_members_status; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_members_status ON organizations.members USING btree (membership_status);


--
-- Name: idx_members_unit; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_members_unit ON organizations.members USING btree (unit_id);


--
-- Name: idx_officers_org; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_officers_org ON organizations.officers USING btree (organization_id);


--
-- Name: idx_officers_role; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_officers_role ON organizations.officers USING btree (role_type_id);


--
-- Name: idx_officers_unit; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_officers_unit ON organizations.officers USING btree (unit_id);


--
-- Name: idx_org_buildings_building; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_org_buildings_building ON organizations.buildings USING btree (building_id);


--
-- Name: idx_org_buildings_org; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_org_buildings_org ON organizations.buildings USING btree (organization_id);


--
-- Name: idx_org_buildings_role; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_org_buildings_role ON organizations.buildings USING btree (building_role);


--
-- Name: idx_organizations_headquarters; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_organizations_headquarters ON organizations.organizations USING btree (headquarters_cell_q, headquarters_cell_r);


--
-- Name: idx_organizations_leader; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_organizations_leader ON organizations.organizations USING btree (leader_unit_id);


--
-- Name: idx_organizations_name; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_organizations_name ON organizations.organizations USING btree (name);


--
-- Name: idx_organizations_parent; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_organizations_parent ON organizations.organizations USING btree (parent_organization_id);


--
-- Name: idx_organizations_type; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_organizations_type ON organizations.organizations USING btree (organization_type_id);


--
-- Name: idx_organizations_zone; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_organizations_zone ON organizations.organizations USING btree (voronoi_zone_id);


--
-- Name: idx_territory_cells_location; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_territory_cells_location ON organizations.territory_cells USING btree (cell_q, cell_r);


--
-- Name: idx_territory_cells_org; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_territory_cells_org ON organizations.territory_cells USING btree (organization_id);


--
-- Name: idx_territory_contours_chunk; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_territory_contours_chunk ON organizations.territory_contours USING btree (chunk_x, chunk_y);


--
-- Name: idx_territory_contours_org; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_territory_contours_org ON organizations.territory_contours USING btree (organization_id);


--
-- Name: idx_treasury_items_instance; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_treasury_items_instance ON organizations.treasury_items USING btree (item_instance_id);


--
-- Name: idx_treasury_items_org; Type: INDEX; Schema: organizations; Owner: living_landz_srv
--

CREATE INDEX idx_treasury_items_org ON organizations.treasury_items USING btree (organization_id);


--
-- Name: idx_item_instances_item; Type: INDEX; Schema: resources; Owner: living_landz_srv
--

CREATE INDEX idx_item_instances_item ON resources.item_instances USING btree (item_id);


--
-- Name: idx_item_instances_owner; Type: INDEX; Schema: resources; Owner: living_landz_srv
--

CREATE INDEX idx_item_instances_owner ON resources.item_instances USING btree (owner_unit_id) WHERE (owner_unit_id IS NOT NULL);


--
-- Name: idx_item_instances_perishable; Type: INDEX; Schema: resources; Owner: living_landz_srv
--

CREATE INDEX idx_item_instances_perishable ON resources.item_instances USING btree (current_decay, last_decay_update) WHERE (current_decay > (0)::numeric);


--
-- Name: idx_item_instances_world_pos; Type: INDEX; Schema: resources; Owner: living_landz_srv
--

CREATE INDEX idx_item_instances_world_pos ON resources.item_instances USING btree (world_chunk_x, world_chunk_y, world_cell_q, world_cell_r) WHERE (owner_unit_id IS NULL);


--
-- Name: idx_item_stat_modifiers; Type: INDEX; Schema: resources; Owner: living_landz_srv
--

CREATE INDEX idx_item_stat_modifiers ON resources.item_stat_modifiers USING btree (item_id);


--
-- Name: idx_items_category; Type: INDEX; Schema: resources; Owner: living_landz_srv
--

CREATE INDEX idx_items_category ON resources.items USING btree (category_id);


--
-- Name: idx_items_craftable; Type: INDEX; Schema: resources; Owner: living_landz_srv
--

CREATE INDEX idx_items_craftable ON resources.items USING btree (is_craftable) WHERE (is_craftable = true);


--
-- Name: idx_items_equipable; Type: INDEX; Schema: resources; Owner: living_landz_srv
--

CREATE INDEX idx_items_equipable ON resources.items USING btree (is_equipable) WHERE (is_equipable = true);


--
-- Name: idx_items_perishable; Type: INDEX; Schema: resources; Owner: living_landz_srv
--

CREATE INDEX idx_items_perishable ON resources.items USING btree (is_perishable) WHERE (is_perishable = true);


--
-- Name: idx_items_type; Type: INDEX; Schema: resources; Owner: living_landz_srv
--

CREATE INDEX idx_items_type ON resources.items USING btree (item_type_id);


--
-- Name: idx_recipe_ingredients_item; Type: INDEX; Schema: resources; Owner: living_landz_srv
--

CREATE INDEX idx_recipe_ingredients_item ON resources.recipe_ingredients USING btree (item_id);


--
-- Name: idx_recipe_ingredients_recipe; Type: INDEX; Schema: resources; Owner: living_landz_srv
--

CREATE INDEX idx_recipe_ingredients_recipe ON resources.recipe_ingredients USING btree (recipe_id);


--
-- Name: idx_recipes_result_item; Type: INDEX; Schema: resources; Owner: living_landz_srv
--

CREATE INDEX idx_recipes_result_item ON resources.recipes USING btree (result_item_id);


--
-- Name: idx_recipes_skill; Type: INDEX; Schema: resources; Owner: living_landz_srv
--

CREATE INDEX idx_recipes_skill ON resources.recipes USING btree (required_skill_id) WHERE (required_skill_id IS NOT NULL);


--
-- Name: idx_resource_types_category; Type: INDEX; Schema: resources; Owner: living_landz_srv
--

CREATE INDEX idx_resource_types_category ON resources.resource_types USING btree (category_id);


--
-- Name: idx_resource_types_specific; Type: INDEX; Schema: resources; Owner: living_landz_srv
--

CREATE INDEX idx_resource_types_specific ON resources.resource_types USING btree (specific_type_id);


--
-- Name: idx_cells_building; Type: INDEX; Schema: terrain; Owner: living_landz_srv
--

CREATE INDEX idx_cells_building ON terrain.cells USING btree (building_id);


--
-- Name: idx_cells_chunk; Type: INDEX; Schema: terrain; Owner: living_landz_srv
--

CREATE INDEX idx_cells_chunk ON terrain.cells USING btree (chunk_x, chunk_y);


--
-- Name: idx_ocean_data_generated; Type: INDEX; Schema: terrain; Owner: living_landz_srv
--

CREATE INDEX idx_ocean_data_generated ON terrain.ocean_data USING btree (generated_at);


--
-- Name: idx_road_cache_generated; Type: INDEX; Schema: terrain; Owner: living_landz_srv
--

CREATE INDEX idx_road_cache_generated ON terrain.road_chunk_cache USING btree (generated_at);


--
-- Name: idx_road_chunk_visibility_chunk; Type: INDEX; Schema: terrain; Owner: living_landz_srv
--

CREATE INDEX idx_road_chunk_visibility_chunk ON terrain.road_chunk_visibility USING btree (chunk_x, chunk_y);


--
-- Name: idx_road_chunk_visibility_segment; Type: INDEX; Schema: terrain; Owner: living_landz_srv
--

CREATE INDEX idx_road_chunk_visibility_segment ON terrain.road_chunk_visibility USING btree (segment_id);


--
-- Name: idx_road_segments_chunk; Type: INDEX; Schema: terrain; Owner: living_landz_srv
--

CREATE INDEX idx_road_segments_chunk ON terrain.road_segments USING btree (chunk_x, chunk_y);


--
-- Name: idx_road_segments_connections; Type: INDEX; Schema: terrain; Owner: living_landz_srv
--

CREATE INDEX idx_road_segments_connections ON terrain.road_segments USING btree (start_q, start_r, end_q, end_r);


--
-- Name: idx_road_segments_end; Type: INDEX; Schema: terrain; Owner: living_landz_srv
--

CREATE INDEX idx_road_segments_end ON terrain.road_segments USING btree (end_q, end_r);


--
-- Name: idx_road_segments_start; Type: INDEX; Schema: terrain; Owner: living_landz_srv
--

CREATE INDEX idx_road_segments_start ON terrain.road_segments USING btree (start_q, start_r);


--
-- Name: idx_terrain_biomes_generated; Type: INDEX; Schema: terrain; Owner: living_landz_srv
--

CREATE INDEX idx_terrain_biomes_generated ON terrain.terrain_biomes USING btree (generated_at);


--
-- Name: idx_terrains_generated; Type: INDEX; Schema: terrain; Owner: living_landz_srv
--

CREATE INDEX idx_terrains_generated ON terrain.terrains USING btree (generated_at);


--
-- Name: idx_voronoi_zone_cells_zone; Type: INDEX; Schema: terrain; Owner: living_landz_srv
--

CREATE INDEX idx_voronoi_zone_cells_zone ON terrain.voronoi_zone_cells USING btree (zone_id);


--
-- Name: idx_voronoi_zones_biome; Type: INDEX; Schema: terrain; Owner: living_landz_srv
--

CREATE INDEX idx_voronoi_zones_biome ON terrain.voronoi_zones USING btree (biome_type);


--
-- Name: idx_voronoi_zones_seed; Type: INDEX; Schema: terrain; Owner: living_landz_srv
--

CREATE INDEX idx_voronoi_zones_seed ON terrain.voronoi_zones USING btree (seed_cell_q, seed_cell_r);


--
-- Name: idx_unique_unit_slot; Type: INDEX; Schema: units; Owner: living_landz_srv
--

CREATE UNIQUE INDEX idx_unique_unit_slot ON units.units USING btree (current_chunk_x, current_chunk_y, current_cell_q, current_cell_r, slot_type, slot_index) WHERE ((slot_type IS NOT NULL) AND (slot_index IS NOT NULL));


--
-- Name: idx_unit_automated_actions_enabled; Type: INDEX; Schema: units; Owner: living_landz_srv
--

CREATE INDEX idx_unit_automated_actions_enabled ON units.unit_automated_actions USING btree (unit_id, is_enabled) WHERE (is_enabled = true);


--
-- Name: idx_unit_automated_actions_unit; Type: INDEX; Schema: units; Owner: living_landz_srv
--

CREATE INDEX idx_unit_automated_actions_unit ON units.unit_automated_actions USING btree (unit_id);


--
-- Name: idx_unit_consumption_unit; Type: INDEX; Schema: units; Owner: living_landz_srv
--

CREATE INDEX idx_unit_consumption_unit ON units.unit_consumption_demands USING btree (unit_id);


--
-- Name: idx_unit_equipment_unit; Type: INDEX; Schema: units; Owner: living_landz_srv
--

CREATE INDEX idx_unit_equipment_unit ON units.unit_equipment USING btree (unit_id);


--
-- Name: idx_unit_inventory_item; Type: INDEX; Schema: units; Owner: living_landz_srv
--

CREATE INDEX idx_unit_inventory_item ON units.unit_inventory USING btree (item_id);


--
-- Name: idx_unit_inventory_unit; Type: INDEX; Schema: units; Owner: living_landz_srv
--

CREATE INDEX idx_unit_inventory_unit ON units.unit_inventory USING btree (unit_id);


--
-- Name: idx_unit_skills_unit; Type: INDEX; Schema: units; Owner: living_landz_srv
--

CREATE INDEX idx_unit_skills_unit ON units.unit_skills USING btree (unit_id);


--
-- Name: idx_units_cell_slot; Type: INDEX; Schema: units; Owner: living_landz_srv
--

CREATE INDEX idx_units_cell_slot ON units.units USING btree (current_chunk_x, current_chunk_y, current_cell_q, current_cell_r, slot_type, slot_index);


--
-- Name: idx_units_one_lord_per_player; Type: INDEX; Schema: units; Owner: living_landz_srv
--

CREATE UNIQUE INDEX idx_units_one_lord_per_player ON units.units USING btree (player_id) WHERE ((is_lord = true) AND (player_id IS NOT NULL));


--
-- Name: idx_units_player; Type: INDEX; Schema: units; Owner: living_landz_srv
--

CREATE INDEX idx_units_player ON units.units USING btree (player_id) WHERE (player_id IS NOT NULL);


--
-- Name: idx_units_position; Type: INDEX; Schema: units; Owner: living_landz_srv
--

CREATE INDEX idx_units_position ON units.units USING btree (current_chunk_x, current_chunk_y, current_cell_q, current_cell_r);


--
-- Name: idx_units_profession; Type: INDEX; Schema: units; Owner: living_landz_srv
--

CREATE INDEX idx_units_profession ON units.units USING btree (profession_id);


--
-- Name: officers trigger_cleanup_leader_on_removal; Type: TRIGGER; Schema: organizations; Owner: living_landz_srv
--

CREATE TRIGGER trigger_cleanup_leader_on_removal BEFORE DELETE ON organizations.officers FOR EACH ROW EXECUTE FUNCTION organizations.cleanup_leader_on_officer_removal();


--
-- Name: members trigger_remove_officer_on_suspension; Type: TRIGGER; Schema: organizations; Owner: living_landz_srv
--

CREATE TRIGGER trigger_remove_officer_on_suspension AFTER UPDATE ON organizations.members FOR EACH ROW WHEN (((old.membership_status)::text IS DISTINCT FROM (new.membership_status)::text)) EXECUTE FUNCTION organizations.remove_officer_on_member_removal();


--
-- Name: organizations trigger_update_organization_timestamp; Type: TRIGGER; Schema: organizations; Owner: living_landz_srv
--

CREATE TRIGGER trigger_update_organization_timestamp BEFORE UPDATE ON organizations.organizations FOR EACH ROW EXECUTE FUNCTION organizations.update_organization_timestamp();


--
-- Name: members trigger_update_population_delete; Type: TRIGGER; Schema: organizations; Owner: living_landz_srv
--

CREATE TRIGGER trigger_update_population_delete AFTER DELETE ON organizations.members FOR EACH ROW EXECUTE FUNCTION organizations.update_organization_population();


--
-- Name: members trigger_update_population_insert; Type: TRIGGER; Schema: organizations; Owner: living_landz_srv
--

CREATE TRIGGER trigger_update_population_insert AFTER INSERT ON organizations.members FOR EACH ROW EXECUTE FUNCTION organizations.update_organization_population();


--
-- Name: members trigger_update_population_update; Type: TRIGGER; Schema: organizations; Owner: living_landz_srv
--

CREATE TRIGGER trigger_update_population_update AFTER UPDATE ON organizations.members FOR EACH ROW WHEN (((old.membership_status)::text IS DISTINCT FROM (new.membership_status)::text)) EXECUTE FUNCTION organizations.update_organization_population();


--
-- Name: territory_cells trigger_update_territory_area_delete; Type: TRIGGER; Schema: organizations; Owner: living_landz_srv
--

CREATE TRIGGER trigger_update_territory_area_delete AFTER DELETE ON organizations.territory_cells FOR EACH ROW EXECUTE FUNCTION organizations.update_territory_area();


--
-- Name: territory_cells trigger_update_territory_area_insert; Type: TRIGGER; Schema: organizations; Owner: living_landz_srv
--

CREATE TRIGGER trigger_update_territory_area_insert AFTER INSERT ON organizations.territory_cells FOR EACH ROW EXECUTE FUNCTION organizations.update_territory_area();


--
-- Name: officers trigger_validate_officer_role; Type: TRIGGER; Schema: organizations; Owner: living_landz_srv
--

CREATE TRIGGER trigger_validate_officer_role BEFORE INSERT OR UPDATE ON organizations.officers FOR EACH ROW EXECUTE FUNCTION organizations.validate_officer_role();


--
-- Name: organizations trigger_validate_organization; Type: TRIGGER; Schema: organizations; Owner: living_landz_srv
--

CREATE TRIGGER trigger_validate_organization BEFORE INSERT OR UPDATE ON organizations.organizations FOR EACH ROW EXECUTE FUNCTION organizations.validate_organization_constraints();


--
-- Name: territory_contours update_territory_contours_updated_at; Type: TRIGGER; Schema: organizations; Owner: living_landz_srv
--

CREATE TRIGGER update_territory_contours_updated_at BEFORE UPDATE ON organizations.territory_contours FOR EACH ROW EXECUTE FUNCTION organizations.update_updated_at_column();


--
-- Name: item_instances trigger_update_item_decay; Type: TRIGGER; Schema: resources; Owner: living_landz_srv
--

CREATE TRIGGER trigger_update_item_decay BEFORE INSERT OR UPDATE ON resources.item_instances FOR EACH ROW EXECUTE FUNCTION resources.update_item_decay();


--
-- Name: voronoi_zone_cells trigger_update_zone_cell_count; Type: TRIGGER; Schema: terrain; Owner: living_landz_srv
--

CREATE TRIGGER trigger_update_zone_cell_count AFTER INSERT OR DELETE ON terrain.voronoi_zone_cells FOR EACH ROW EXECUTE FUNCTION public.update_zone_cell_count();


--
-- Name: unit_automated_actions update_unit_automated_actions_updated_at; Type: TRIGGER; Schema: units; Owner: living_landz_srv
--

CREATE TRIGGER update_unit_automated_actions_updated_at BEFORE UPDATE ON units.unit_automated_actions FOR EACH ROW EXECUTE FUNCTION units.update_updated_at_column();


--
-- Name: unit_base_stats update_unit_base_stats_updated_at; Type: TRIGGER; Schema: units; Owner: living_landz_srv
--

CREATE TRIGGER update_unit_base_stats_updated_at BEFORE UPDATE ON units.unit_base_stats FOR EACH ROW EXECUTE FUNCTION units.update_updated_at_column();


--
-- Name: unit_derived_stats update_unit_derived_stats_updated_at; Type: TRIGGER; Schema: units; Owner: living_landz_srv
--

CREATE TRIGGER update_unit_derived_stats_updated_at BEFORE UPDATE ON units.unit_derived_stats FOR EACH ROW EXECUTE FUNCTION units.update_updated_at_column();


--
-- Name: unit_inventory update_unit_inventory_updated_at; Type: TRIGGER; Schema: units; Owner: living_landz_srv
--

CREATE TRIGGER update_unit_inventory_updated_at BEFORE UPDATE ON units.unit_inventory FOR EACH ROW EXECUTE FUNCTION units.update_updated_at_column();


--
-- Name: unit_skills update_unit_skills_updated_at; Type: TRIGGER; Schema: units; Owner: living_landz_srv
--

CREATE TRIGGER update_unit_skills_updated_at BEFORE UPDATE ON units.unit_skills FOR EACH ROW EXECUTE FUNCTION units.update_updated_at_column();


--
-- Name: units update_units_updated_at; Type: TRIGGER; Schema: units; Owner: living_landz_srv
--

CREATE TRIGGER update_units_updated_at BEFORE UPDATE ON units.units FOR EACH ROW EXECUTE FUNCTION units.update_updated_at_column();


--
-- Name: build_building_actions build_building_actions_action_id_fkey; Type: FK CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.build_building_actions
    ADD CONSTRAINT build_building_actions_action_id_fkey FOREIGN KEY (action_id) REFERENCES actions.scheduled_actions(id) ON DELETE CASCADE;


--
-- Name: build_building_actions build_building_actions_building_type_id_fkey; Type: FK CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.build_building_actions
    ADD CONSTRAINT build_building_actions_building_type_id_fkey FOREIGN KEY (building_type_id) REFERENCES buildings.building_types(id);


--
-- Name: build_road_actions build_road_actions_action_id_fkey; Type: FK CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.build_road_actions
    ADD CONSTRAINT build_road_actions_action_id_fkey FOREIGN KEY (action_id) REFERENCES actions.scheduled_actions(id) ON DELETE CASCADE;


--
-- Name: craft_resource_actions craft_resource_actions_action_id_fkey; Type: FK CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.craft_resource_actions
    ADD CONSTRAINT craft_resource_actions_action_id_fkey FOREIGN KEY (action_id) REFERENCES actions.scheduled_actions(id) ON DELETE CASCADE;


--
-- Name: harvest_resource_actions harvest_resource_actions_action_id_fkey; Type: FK CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.harvest_resource_actions
    ADD CONSTRAINT harvest_resource_actions_action_id_fkey FOREIGN KEY (action_id) REFERENCES actions.scheduled_actions(id) ON DELETE CASCADE;


--
-- Name: harvest_resource_actions harvest_resource_actions_resource_type_id_fkey; Type: FK CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.harvest_resource_actions
    ADD CONSTRAINT harvest_resource_actions_resource_type_id_fkey FOREIGN KEY (resource_type_id) REFERENCES resources.resource_types(id);


--
-- Name: move_unit_actions move_unit_actions_action_id_fkey; Type: FK CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.move_unit_actions
    ADD CONSTRAINT move_unit_actions_action_id_fkey FOREIGN KEY (action_id) REFERENCES actions.scheduled_actions(id) ON DELETE CASCADE;


--
-- Name: scheduled_actions scheduled_actions_action_specific_type_id_fkey; Type: FK CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.scheduled_actions
    ADD CONSTRAINT scheduled_actions_action_specific_type_id_fkey FOREIGN KEY (action_specific_type_id) REFERENCES actions.action_specific_types(id);


--
-- Name: scheduled_actions scheduled_actions_action_type_id_fkey; Type: FK CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.scheduled_actions
    ADD CONSTRAINT scheduled_actions_action_type_id_fkey FOREIGN KEY (action_type_id) REFERENCES actions.action_types(id);


--
-- Name: scheduled_actions scheduled_actions_status_id_fkey; Type: FK CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.scheduled_actions
    ADD CONSTRAINT scheduled_actions_status_id_fkey FOREIGN KEY (status_id) REFERENCES actions.action_statuses(id);


--
-- Name: send_message_actions send_message_actions_action_id_fkey; Type: FK CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.send_message_actions
    ADD CONSTRAINT send_message_actions_action_id_fkey FOREIGN KEY (action_id) REFERENCES actions.scheduled_actions(id) ON DELETE CASCADE;


--
-- Name: send_message_receivers send_message_receivers_action_id_fkey; Type: FK CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.send_message_receivers
    ADD CONSTRAINT send_message_receivers_action_id_fkey FOREIGN KEY (action_id) REFERENCES actions.send_message_actions(action_id) ON DELETE CASCADE;


--
-- Name: train_unit_actions train_unit_actions_action_id_fkey; Type: FK CONSTRAINT; Schema: actions; Owner: living_landz_srv
--

ALTER TABLE ONLY actions.train_unit_actions
    ADD CONSTRAINT train_unit_actions_action_id_fkey FOREIGN KEY (action_id) REFERENCES actions.scheduled_actions(id) ON DELETE CASCADE;


--
-- Name: agriculture agriculture_agriculture_type_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.agriculture
    ADD CONSTRAINT agriculture_agriculture_type_id_fkey FOREIGN KEY (agriculture_type_id) REFERENCES buildings.agriculture_types(id);


--
-- Name: agriculture agriculture_building_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.agriculture
    ADD CONSTRAINT agriculture_building_id_fkey FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE;


--
-- Name: animal_breeding animal_breeding_animal_type_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.animal_breeding
    ADD CONSTRAINT animal_breeding_animal_type_id_fkey FOREIGN KEY (animal_type_id) REFERENCES buildings.animal_breeding_types(id);


--
-- Name: animal_breeding animal_breeding_building_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.animal_breeding
    ADD CONSTRAINT animal_breeding_building_id_fkey FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE;


--
-- Name: building_types building_types_category_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.building_types
    ADD CONSTRAINT building_types_category_id_fkey FOREIGN KEY (category_id) REFERENCES buildings.building_categories(id);


--
-- Name: building_types building_types_specific_type_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.building_types
    ADD CONSTRAINT building_types_specific_type_id_fkey FOREIGN KEY (specific_type_id) REFERENCES buildings.building_specific_types(id);


--
-- Name: buildings_base buildings_base_building_type_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.buildings_base
    ADD CONSTRAINT buildings_base_building_type_id_fkey FOREIGN KEY (building_type_id) REFERENCES buildings.building_types(id);


--
-- Name: buildings_base buildings_base_category_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.buildings_base
    ADD CONSTRAINT buildings_base_category_id_fkey FOREIGN KEY (category_id) REFERENCES buildings.building_categories(id);


--
-- Name: commerce commerce_building_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.commerce
    ADD CONSTRAINT commerce_building_id_fkey FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE;


--
-- Name: commerce commerce_commerce_type_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.commerce
    ADD CONSTRAINT commerce_commerce_type_id_fkey FOREIGN KEY (commerce_type_id) REFERENCES buildings.commerce_types(id);


--
-- Name: construction_costs construction_costs_building_type_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.construction_costs
    ADD CONSTRAINT construction_costs_building_type_id_fkey FOREIGN KEY (building_type_id) REFERENCES buildings.building_types(id) ON DELETE CASCADE;


--
-- Name: construction_costs construction_costs_item_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.construction_costs
    ADD CONSTRAINT construction_costs_item_id_fkey FOREIGN KEY (item_id) REFERENCES resources.items(id);


--
-- Name: cult cult_building_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.cult
    ADD CONSTRAINT cult_building_id_fkey FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE;


--
-- Name: cult cult_cult_type_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.cult
    ADD CONSTRAINT cult_cult_type_id_fkey FOREIGN KEY (cult_type_id) REFERENCES buildings.cult_types(id);


--
-- Name: entertainment entertainment_building_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.entertainment
    ADD CONSTRAINT entertainment_building_id_fkey FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE;


--
-- Name: entertainment entertainment_entertainment_type_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.entertainment
    ADD CONSTRAINT entertainment_entertainment_type_id_fkey FOREIGN KEY (entertainment_type_id) REFERENCES buildings.entertainment_types(id);


--
-- Name: manufacturing_workshops manufacturing_workshops_building_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.manufacturing_workshops
    ADD CONSTRAINT manufacturing_workshops_building_id_fkey FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE;


--
-- Name: manufacturing_workshops manufacturing_workshops_workshop_type_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.manufacturing_workshops
    ADD CONSTRAINT manufacturing_workshops_workshop_type_id_fkey FOREIGN KEY (workshop_type_id) REFERENCES buildings.manufacturing_workshop_types(id);


--
-- Name: trees trees_building_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.trees
    ADD CONSTRAINT trees_building_id_fkey FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE;


--
-- Name: trees trees_tree_type_id_fkey; Type: FK CONSTRAINT; Schema: buildings; Owner: living_landz_srv
--

ALTER TABLE ONLY buildings.trees
    ADD CONSTRAINT trees_tree_type_id_fkey FOREIGN KEY (tree_type_id) REFERENCES buildings.tree_types(id);


--
-- Name: characters characters_coat_of_arms_id_fkey; Type: FK CONSTRAINT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.characters
    ADD CONSTRAINT characters_coat_of_arms_id_fkey FOREIGN KEY (coat_of_arms_id) REFERENCES game.coats_of_arms(id) ON DELETE SET NULL;


--
-- Name: characters characters_player_id_fkey; Type: FK CONSTRAINT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.characters
    ADD CONSTRAINT characters_player_id_fkey FOREIGN KEY (player_id) REFERENCES game.players(id) ON DELETE CASCADE;


--
-- Name: players players_coat_of_arms_id_fkey; Type: FK CONSTRAINT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.players
    ADD CONSTRAINT players_coat_of_arms_id_fkey FOREIGN KEY (coat_of_arms_id) REFERENCES game.coats_of_arms(id) ON DELETE SET NULL;


--
-- Name: players players_language_id_fkey; Type: FK CONSTRAINT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.players
    ADD CONSTRAINT players_language_id_fkey FOREIGN KEY (language_id) REFERENCES game.languages(id);


--
-- Name: translations translations_language_id_fkey; Type: FK CONSTRAINT; Schema: game; Owner: living_landz_srv
--

ALTER TABLE ONLY game.translations
    ADD CONSTRAINT translations_language_id_fkey FOREIGN KEY (language_id) REFERENCES game.languages(id);


--
-- Name: buildings buildings_acquired_by_unit_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.buildings
    ADD CONSTRAINT buildings_acquired_by_unit_id_fkey FOREIGN KEY (acquired_by_unit_id) REFERENCES units.units(id) ON DELETE SET NULL;


--
-- Name: buildings buildings_organization_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.buildings
    ADD CONSTRAINT buildings_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES organizations.organizations(id) ON DELETE CASCADE;


--
-- Name: diplomatic_relations diplomatic_relations_established_by_unit_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.diplomatic_relations
    ADD CONSTRAINT diplomatic_relations_established_by_unit_id_fkey FOREIGN KEY (established_by_unit_id) REFERENCES units.units(id) ON DELETE SET NULL;


--
-- Name: diplomatic_relations diplomatic_relations_organization_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.diplomatic_relations
    ADD CONSTRAINT diplomatic_relations_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES organizations.organizations(id) ON DELETE CASCADE;


--
-- Name: diplomatic_relations diplomatic_relations_target_organization_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.diplomatic_relations
    ADD CONSTRAINT diplomatic_relations_target_organization_id_fkey FOREIGN KEY (target_organization_id) REFERENCES organizations.organizations(id) ON DELETE CASCADE;


--
-- Name: members members_invited_by_unit_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.members
    ADD CONSTRAINT members_invited_by_unit_id_fkey FOREIGN KEY (invited_by_unit_id) REFERENCES units.units(id) ON DELETE SET NULL;


--
-- Name: members members_organization_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.members
    ADD CONSTRAINT members_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES organizations.organizations(id) ON DELETE CASCADE;


--
-- Name: members members_unit_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.members
    ADD CONSTRAINT members_unit_id_fkey FOREIGN KEY (unit_id) REFERENCES units.units(id) ON DELETE CASCADE;


--
-- Name: officers officers_appointed_by_unit_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.officers
    ADD CONSTRAINT officers_appointed_by_unit_id_fkey FOREIGN KEY (appointed_by_unit_id) REFERENCES units.units(id) ON DELETE SET NULL;


--
-- Name: officers officers_organization_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.officers
    ADD CONSTRAINT officers_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES organizations.organizations(id) ON DELETE CASCADE;


--
-- Name: officers officers_role_type_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.officers
    ADD CONSTRAINT officers_role_type_id_fkey FOREIGN KEY (role_type_id) REFERENCES organizations.role_types(id);


--
-- Name: officers officers_unit_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.officers
    ADD CONSTRAINT officers_unit_id_fkey FOREIGN KEY (unit_id) REFERENCES units.units(id) ON DELETE CASCADE;


--
-- Name: organization_role_compatibility organization_role_compatibility_organization_type_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.organization_role_compatibility
    ADD CONSTRAINT organization_role_compatibility_organization_type_id_fkey FOREIGN KEY (organization_type_id) REFERENCES organizations.organization_types(id);


--
-- Name: organization_role_compatibility organization_role_compatibility_role_type_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.organization_role_compatibility
    ADD CONSTRAINT organization_role_compatibility_role_type_id_fkey FOREIGN KEY (role_type_id) REFERENCES organizations.role_types(id);


--
-- Name: organizations organizations_leader_unit_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.organizations
    ADD CONSTRAINT organizations_leader_unit_id_fkey FOREIGN KEY (leader_unit_id) REFERENCES units.units(id) ON DELETE SET NULL;


--
-- Name: organizations organizations_organization_type_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.organizations
    ADD CONSTRAINT organizations_organization_type_id_fkey FOREIGN KEY (organization_type_id) REFERENCES organizations.organization_types(id);


--
-- Name: organizations organizations_parent_organization_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.organizations
    ADD CONSTRAINT organizations_parent_organization_id_fkey FOREIGN KEY (parent_organization_id) REFERENCES organizations.organizations(id) ON DELETE SET NULL;


--
-- Name: organizations organizations_voronoi_zone_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.organizations
    ADD CONSTRAINT organizations_voronoi_zone_id_fkey FOREIGN KEY (voronoi_zone_id) REFERENCES terrain.voronoi_zones(id) ON DELETE SET NULL;


--
-- Name: territory_cells territory_cells_organization_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.territory_cells
    ADD CONSTRAINT territory_cells_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES organizations.organizations(id) ON DELETE CASCADE;


--
-- Name: territory_contours territory_contours_organization_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.territory_contours
    ADD CONSTRAINT territory_contours_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES organizations.organizations(id) ON DELETE CASCADE;


--
-- Name: treasury_items treasury_items_item_instance_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.treasury_items
    ADD CONSTRAINT treasury_items_item_instance_id_fkey FOREIGN KEY (item_instance_id) REFERENCES resources.item_instances(id) ON DELETE CASCADE;


--
-- Name: treasury_items treasury_items_organization_id_fkey; Type: FK CONSTRAINT; Schema: organizations; Owner: living_landz_srv
--

ALTER TABLE ONLY organizations.treasury_items
    ADD CONSTRAINT treasury_items_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES organizations.organizations(id) ON DELETE CASCADE;


--
-- Name: harvest_yields harvest_yields_required_tool_item_id_fkey; Type: FK CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.harvest_yields
    ADD CONSTRAINT harvest_yields_required_tool_item_id_fkey FOREIGN KEY (required_tool_item_id) REFERENCES resources.items(id);


--
-- Name: harvest_yields harvest_yields_resource_specific_type_id_fkey; Type: FK CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.harvest_yields
    ADD CONSTRAINT harvest_yields_resource_specific_type_id_fkey FOREIGN KEY (resource_specific_type_id) REFERENCES resources.resource_specific_types(id);


--
-- Name: harvest_yields harvest_yields_result_item_id_fkey; Type: FK CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.harvest_yields
    ADD CONSTRAINT harvest_yields_result_item_id_fkey FOREIGN KEY (result_item_id) REFERENCES resources.items(id);


--
-- Name: item_instances item_instances_item_id_fkey; Type: FK CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.item_instances
    ADD CONSTRAINT item_instances_item_id_fkey FOREIGN KEY (item_id) REFERENCES resources.items(id);


--
-- Name: item_stat_modifiers item_stat_modifiers_item_id_fkey; Type: FK CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.item_stat_modifiers
    ADD CONSTRAINT item_stat_modifiers_item_id_fkey FOREIGN KEY (item_id) REFERENCES resources.items(id) ON DELETE CASCADE;


--
-- Name: items items_category_id_fkey; Type: FK CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.items
    ADD CONSTRAINT items_category_id_fkey FOREIGN KEY (category_id) REFERENCES resources.resource_categories(id);


--
-- Name: items items_equipment_slot_id_fkey; Type: FK CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.items
    ADD CONSTRAINT items_equipment_slot_id_fkey FOREIGN KEY (equipment_slot_id) REFERENCES resources.equipment_slots(id);


--
-- Name: items items_item_type_id_fkey; Type: FK CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.items
    ADD CONSTRAINT items_item_type_id_fkey FOREIGN KEY (item_type_id) REFERENCES resources.item_types(id);


--
-- Name: recipe_ingredients recipe_ingredients_item_id_fkey; Type: FK CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.recipe_ingredients
    ADD CONSTRAINT recipe_ingredients_item_id_fkey FOREIGN KEY (item_id) REFERENCES resources.items(id);


--
-- Name: recipe_ingredients recipe_ingredients_recipe_id_fkey; Type: FK CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.recipe_ingredients
    ADD CONSTRAINT recipe_ingredients_recipe_id_fkey FOREIGN KEY (recipe_id) REFERENCES resources.recipes(id) ON DELETE CASCADE;


--
-- Name: recipes recipes_result_item_id_fkey; Type: FK CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.recipes
    ADD CONSTRAINT recipes_result_item_id_fkey FOREIGN KEY (result_item_id) REFERENCES resources.items(id);


--
-- Name: resource_specific_types resource_specific_types_category_id_fkey; Type: FK CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.resource_specific_types
    ADD CONSTRAINT resource_specific_types_category_id_fkey FOREIGN KEY (category_id) REFERENCES resources.resource_categories(id);


--
-- Name: resource_types resource_types_category_id_fkey; Type: FK CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.resource_types
    ADD CONSTRAINT resource_types_category_id_fkey FOREIGN KEY (category_id) REFERENCES resources.resource_categories(id);


--
-- Name: resource_types resource_types_specific_type_id_fkey; Type: FK CONSTRAINT; Schema: resources; Owner: living_landz_srv
--

ALTER TABLE ONLY resources.resource_types
    ADD CONSTRAINT resource_types_specific_type_id_fkey FOREIGN KEY (specific_type_id) REFERENCES resources.resource_specific_types(id);


--
-- Name: cells cells_biome_id_fkey; Type: FK CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.cells
    ADD CONSTRAINT cells_biome_id_fkey FOREIGN KEY (biome_id) REFERENCES terrain.biome_types(id);


--
-- Name: cells cells_building_id_fkey; Type: FK CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.cells
    ADD CONSTRAINT cells_building_id_fkey FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE SET NULL;


--
-- Name: road_chunk_visibility road_chunk_visibility_segment_id_fkey; Type: FK CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.road_chunk_visibility
    ADD CONSTRAINT road_chunk_visibility_segment_id_fkey FOREIGN KEY (segment_id) REFERENCES terrain.road_segments(id) ON DELETE CASCADE;


--
-- Name: road_segments road_segments_end_q_end_r_fkey; Type: FK CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.road_segments
    ADD CONSTRAINT road_segments_end_q_end_r_fkey FOREIGN KEY (end_q, end_r) REFERENCES terrain.cells(q, r) ON DELETE CASCADE;


--
-- Name: road_segments road_segments_road_type_id_fkey; Type: FK CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.road_segments
    ADD CONSTRAINT road_segments_road_type_id_fkey FOREIGN KEY (road_type_id) REFERENCES terrain.road_types(id);


--
-- Name: road_segments road_segments_start_q_start_r_fkey; Type: FK CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.road_segments
    ADD CONSTRAINT road_segments_start_q_start_r_fkey FOREIGN KEY (start_q, start_r) REFERENCES terrain.cells(q, r) ON DELETE CASCADE;


--
-- Name: road_types road_types_category_id_fkey; Type: FK CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.road_types
    ADD CONSTRAINT road_types_category_id_fkey FOREIGN KEY (category_id) REFERENCES terrain.road_categories(id);


--
-- Name: terrain_biomes terrain_biomes_biome_id_fkey; Type: FK CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.terrain_biomes
    ADD CONSTRAINT terrain_biomes_biome_id_fkey FOREIGN KEY (biome_id) REFERENCES terrain.biome_types(id);


--
-- Name: voronoi_zone_cells voronoi_zone_cells_zone_id_fkey; Type: FK CONSTRAINT; Schema: terrain; Owner: living_landz_srv
--

ALTER TABLE ONLY terrain.voronoi_zone_cells
    ADD CONSTRAINT voronoi_zone_cells_zone_id_fkey FOREIGN KEY (zone_id) REFERENCES terrain.voronoi_zones(id) ON DELETE CASCADE;


--
-- Name: profession_skill_bonuses profession_skill_bonuses_profession_id_fkey; Type: FK CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.profession_skill_bonuses
    ADD CONSTRAINT profession_skill_bonuses_profession_id_fkey FOREIGN KEY (profession_id) REFERENCES units.professions(id);


--
-- Name: profession_skill_bonuses profession_skill_bonuses_skill_id_fkey; Type: FK CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.profession_skill_bonuses
    ADD CONSTRAINT profession_skill_bonuses_skill_id_fkey FOREIGN KEY (skill_id) REFERENCES units.skills(id);


--
-- Name: unit_automated_actions unit_automated_actions_unit_id_fkey; Type: FK CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_automated_actions
    ADD CONSTRAINT unit_automated_actions_unit_id_fkey FOREIGN KEY (unit_id) REFERENCES units.units(id) ON DELETE CASCADE;


--
-- Name: unit_base_stats unit_base_stats_unit_id_fkey; Type: FK CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_base_stats
    ADD CONSTRAINT unit_base_stats_unit_id_fkey FOREIGN KEY (unit_id) REFERENCES units.units(id) ON DELETE CASCADE;


--
-- Name: unit_consumption_demands unit_consumption_demands_item_id_fkey; Type: FK CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_consumption_demands
    ADD CONSTRAINT unit_consumption_demands_item_id_fkey FOREIGN KEY (item_id) REFERENCES resources.items(id);


--
-- Name: CONSTRAINT unit_consumption_demands_item_id_fkey ON unit_consumption_demands; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON CONSTRAINT unit_consumption_demands_item_id_fkey ON units.unit_consumption_demands IS 'References items from resources.items table';


--
-- Name: unit_consumption_demands unit_consumption_demands_unit_id_fkey; Type: FK CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_consumption_demands
    ADD CONSTRAINT unit_consumption_demands_unit_id_fkey FOREIGN KEY (unit_id) REFERENCES units.units(id) ON DELETE CASCADE;


--
-- Name: unit_derived_stats unit_derived_stats_unit_id_fkey; Type: FK CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_derived_stats
    ADD CONSTRAINT unit_derived_stats_unit_id_fkey FOREIGN KEY (unit_id) REFERENCES units.units(id) ON DELETE CASCADE;


--
-- Name: unit_equipment unit_equipment_equipment_slot_id_fkey; Type: FK CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_equipment
    ADD CONSTRAINT unit_equipment_equipment_slot_id_fkey FOREIGN KEY (equipment_slot_id) REFERENCES resources.equipment_slots(id);


--
-- Name: unit_equipment unit_equipment_item_id_fkey; Type: FK CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_equipment
    ADD CONSTRAINT unit_equipment_item_id_fkey FOREIGN KEY (item_id) REFERENCES resources.items(id);


--
-- Name: CONSTRAINT unit_equipment_item_id_fkey ON unit_equipment; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON CONSTRAINT unit_equipment_item_id_fkey ON units.unit_equipment IS 'References items from resources.items table';


--
-- Name: unit_equipment unit_equipment_unit_id_fkey; Type: FK CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_equipment
    ADD CONSTRAINT unit_equipment_unit_id_fkey FOREIGN KEY (unit_id) REFERENCES units.units(id) ON DELETE CASCADE;


--
-- Name: unit_inventory unit_inventory_item_id_fkey; Type: FK CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_inventory
    ADD CONSTRAINT unit_inventory_item_id_fkey FOREIGN KEY (item_id) REFERENCES resources.items(id);


--
-- Name: CONSTRAINT unit_inventory_item_id_fkey ON unit_inventory; Type: COMMENT; Schema: units; Owner: living_landz_srv
--

COMMENT ON CONSTRAINT unit_inventory_item_id_fkey ON units.unit_inventory IS 'References items from resources.items table';


--
-- Name: unit_inventory unit_inventory_unit_id_fkey; Type: FK CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_inventory
    ADD CONSTRAINT unit_inventory_unit_id_fkey FOREIGN KEY (unit_id) REFERENCES units.units(id) ON DELETE CASCADE;


--
-- Name: unit_skills unit_skills_skill_id_fkey; Type: FK CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_skills
    ADD CONSTRAINT unit_skills_skill_id_fkey FOREIGN KEY (skill_id) REFERENCES units.skills(id);


--
-- Name: unit_skills unit_skills_unit_id_fkey; Type: FK CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.unit_skills
    ADD CONSTRAINT unit_skills_unit_id_fkey FOREIGN KEY (unit_id) REFERENCES units.units(id) ON DELETE CASCADE;


--
-- Name: units units_profession_id_fkey; Type: FK CONSTRAINT; Schema: units; Owner: living_landz_srv
--

ALTER TABLE ONLY units.units
    ADD CONSTRAINT units_profession_id_fkey FOREIGN KEY (profession_id) REFERENCES units.professions(id);


--
-- Name: SCHEMA public; Type: ACL; Schema: -; Owner: pg_database_owner
--

GRANT ALL ON SCHEMA public TO living_landz_srv;


--
-- PostgreSQL database dump complete
--

\unrestrict WdS0PDWDAvqS22er524zGwjOV6Dq7pMCs5lVzTreYPxP0bB05GLXDJktRrkinrc

