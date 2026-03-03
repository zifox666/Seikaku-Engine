# SDE SQLITE 数据库结构

    1. dgmAttributeCategories

        ```sql
        create table main.dgmAttributeCategories
            (
                categoryID          integer not null
                    primary key,
                categoryName        varchar(50)  default NULL,
                categoryDescription varchar(200) default NULL
            );
        ```

    2. dgmAttributeTypes

        ```sql
        create table main.dgmAttributeTypes
            (
                attributeID   integer not null
                    primary key,
                attributeName varchar(100)  default NULL,
                description   varchar(1000) default NULL,
                iconID        integer       default NULL,
                defaultValue  float         default NULL,
                published     integer       default NULL,
                displayName   varchar(150)  default NULL,
                unitID        integer       default NULL,
                stackable     integer       default NULL,
                highIsGood    integer       default NULL,
                categoryID    integer       default NULL,
                constraint dat_hig
                    check (`highIsGood` in (0, 1)),
                constraint dat_pub
                    check (`published` in (0, 1)),
                constraint dat_stack
                    check (`stackable` in (0, 1))
            );
        ```

    3. 

        ```sql
        create table main.dgmEffects
            (
                effectID                       integer not null
                    primary key,
                effectName                     varchar(400)  default NULL,
                effectCategory                 integer       default NULL,
                preExpression                  integer       default NULL,
                postExpression                 integer       default NULL,
                description                    varchar(1000) default NULL,
                guid                           varchar(60)   default NULL,
                iconID                         integer       default NULL,
                isOffensive                    integer       default NULL,
                isAssistance                   integer       default NULL,
                durationAttributeID            integer       default NULL,
                trackingSpeedAttributeID       integer       default NULL,
                dischargeAttributeID           integer       default NULL,
                rangeAttributeID               integer       default NULL,
                falloffAttributeID             integer       default NULL,
                disallowAutoRepeat             integer       default NULL,
                published                      integer       default NULL,
                displayName                    varchar(100)  default NULL,
                isWarpSafe                     integer       default NULL,
                rangeChance                    integer       default NULL,
                electronicChance               integer       default NULL,
                propulsionChance               integer       default NULL,
                distribution                   integer       default NULL,
                sfxName                        varchar(20)   default NULL,
                npcUsageChanceAttributeID      integer       default NULL,
                npcActivationChanceAttributeID integer       default NULL,
                fittingUsageChanceAttributeID  integer       default NULL,
                modifierInfo                   text          default NULL,
                constraint de_assist
                    check (`isAssistance` in (0, 1)),
                constraint de_disallowar
                    check (`disallowAutoRepeat` in (0, 1)),
                constraint de_elecchance
                    check (`electronicChance` in (0, 1)),
                constraint de_offense
                    check (`isOffensive` in (0, 1)),
                constraint de_propchance
                    check (`propulsionChance` in (0, 1)),
                constraint de_published
                    check (`published` in (0, 1)),
                constraint de_rangechance
                    check (`rangeChance` in (0, 1)),
                constraint de_warpsafe
                    check (`isWarpSafe` in (0, 1))
            );
        ```
    
    4. dgmTypeAttributes

        ```sql
        create table main.dgmTypeAttributes
        (
            typeID      integer not null,
            attributeID integer not null,
            valueInt    integer default NULL,
            valueFloat  float   default NULL,
            primary key (typeID, attributeID)
        );

        create index main.idx_dgmTypeAttributes_ix_dgmTypeAttributes_attributeID
            on main.dgmTypeAttributes (attributeID);
        ```

    5. dgmTypeEffects

        ```sql
        create table main.dgmTypeEffects
        (
            typeID    integer not null,
            effectID  integer not null,
            isDefault integer default NULL,
            primary key (typeID, effectID),
            constraint dte_default
                check (`isDefault` in (0, 1))
        );
        ```

    6. eveUnit

        ```sql
        create table main.eveUnits
        (
            unitID      integer not null
                primary key,
            unitName    varchar(100)  default NULL,
            displayName varchar(50)   default NULL,
            description varchar(1000) default NULL
        );
        ```
    
    7. invFlags

        ```sql
        create table main.invFlags
        (
            flagID   integer not null
                primary key,
            flagName varchar(200) default NULL,
            flagText varchar(100) default NULL,
            orderID  integer      default NULL
        );

        ```

    8. invGroups

        ```sql
        create table main.invGroups
        (
            groupID              integer not null
                primary key,
            categoryID           integer      default NULL,
            groupName            varchar(100) default NULL,
            iconID               integer      default NULL,
            useBasePrice         integer      default NULL,
            anchored             integer      default NULL,
            anchorable           integer      default NULL,
            fittableNonSingleton integer      default NULL,
            published            integer      default NULL,
            constraint invgroup_anchorable
                check (`anchorable` in (0, 1)),
            constraint invgroup_anchored
                check (`anchored` in (0, 1)),
            constraint invgroup_fitnonsingle
                check (`fittableNonSingleton` in (0, 1)),
            constraint invgroup_published
                check (`published` in (0, 1)),
            constraint invgroup_usebaseprice
                check (`useBasePrice` in (0, 1))
        );

        create index main.idx_invGroups_ix_invGroups_categoryID
            on main.invGroups (categoryID);

        ```

    9. 
    create table main.invMarketGroups
(
    marketGroupID   integer not null
        primary key,
    parentGroupID   integer       default NULL,
    marketGroupName varchar(100)  default NULL,
    description     varchar(3000) default NULL,
    iconID          integer       default NULL,
    hasTypes        integer       default NULL,
    constraint invmarketgroups_hastypes
        check (`hasTypes` in (0, 1))
);

create table main.invMetaGroups
(
    metaGroupID   integer not null
        primary key,
    metaGroupName varchar(100)  default NULL,
    description   varchar(1000) default NULL,
    iconID        integer       default NULL
);

create table main.invTypes
(
    typeID        integer not null
        primary key,
    groupID       integer        default NULL,
    typeName      varchar(100)   default NULL,
    description   text           default NULL,
    mass          double         default NULL,
    volume        double         default NULL,
    capacity      double         default NULL,
    portionSize   integer        default NULL,
    raceID        integer        default NULL,
    basePrice     decimal(19, 4) default NULL,
    published     integer        default NULL,
    marketGroupID integer        default NULL,
    iconID        integer        default NULL,
    soundID       integer        default NULL,
    graphicID     integer        default NULL,
    constraint invtype_published
        check (`published` in (0, 1))
);

create index main.idx_invTypes_ix_invTypes_groupID
    on main.invTypes (groupID);

create table main.trnTranslations
(
    tcID       integer     not null,
    keyID      integer     not null,
    languageID varchar(50) not null,
    text       text        not null,
    primary key (tcID, keyID, languageID)
);

TC ID	Description
6	invCategories.categoryName
7	invGroups.groupName
8	invTypes.typeName
33	invTypes.description
34	invMetaGroups.description
35	invMetaGroups.metaGroupName
36	invMarketGroups.marketGroupName
40	mapSolarSystems.name
41	mapConstellations.name
42	mapRegions.name