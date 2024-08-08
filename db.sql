-- -------------------------------------------------------------
-- TablePlus 5.3.0(486)
--
-- https://tableplus.com/
--
-- Database: monitorink
-- Generation Time: 2023-02-14 08:47:51.3570
-- -------------------------------------------------------------


-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."magic_link" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "email" varchar NOT NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "state" varchar NOT NULL DEFAULT 'sent'::character varying,
    "token" uuid NOT NULL,
    PRIMARY KEY ("id")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."mesh" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "name" varchar NOT NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "user_id" uuid NOT NULL,
    PRIMARY KEY ("id")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."mesh_access" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "mesh_id" uuid NOT NULL,
    "user_id" uuid,
    "can_edit" bool NOT NULL DEFAULT false,
    "email" varchar NOT NULL,
    PRIMARY KEY ("id")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."resource_state" (
    "server_id" uuid NOT NULL,
    "updated_at" timestamptz NOT NULL DEFAULT now(),
    "t" varchar NOT NULL,
    "device_id" varchar NOT NULL,
    "key" varchar NOT NULL,
    "value" int8 NOT NULL,
    "min" int8 NOT NULL
);
CREATE UNIQUE INDEX "resource_unique" ON "public"."resource_state" USING BTREE ("server_id","t","device_id","key","min");

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."server" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "name" varchar NOT NULL,
    "user_id" uuid NOT NULL DEFAULT 'e58c9a52-a655-4822-853f-11148e178404'::uuid,
    "os" varchar NOT NULL,
    "version" varchar NOT NULL,
    "kernel" varchar NOT NULL,
    "ip" varchar NOT NULL,
    PRIMARY KEY ("id")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."user" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "email" varchar NOT NULL,
    "name" varchar NOT NULL DEFAULT ''::character varying,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "registered_at" timestamptz,
    PRIMARY KEY ("id")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."website" (
    "url" varchar NOT NULL,
    "keyword" varchar NOT NULL,
    "tags" varchar NOT NULL,
    "last_checked" timestamptz,
    "user_id" uuid NOT NULL,
    "useragent" varchar NOT NULL DEFAULT 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36'::character varying,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "paused" bool NOT NULL DEFAULT false,
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    PRIMARY KEY ("id")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."website_state" (
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "state" bool NOT NULL,
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "website_id" uuid NOT NULL,
    PRIMARY KEY ("id")
);

ALTER TABLE "public"."mesh" ADD FOREIGN KEY ("user_id") REFERENCES "public"."user"("id");
ALTER TABLE "public"."mesh_access" ADD FOREIGN KEY ("user_id") REFERENCES "public"."user"("id");
ALTER TABLE "public"."mesh_access" ADD FOREIGN KEY ("mesh_id") REFERENCES "public"."mesh"("id");
ALTER TABLE "public"."website" ADD FOREIGN KEY ("user_id") REFERENCES "public"."user"("id");
ALTER TABLE "public"."website_state" ADD FOREIGN KEY ("website_id") REFERENCES "public"."website"("id");
