truncate table public.menu restart identity;

INSERT INTO "public"."menu" ("id", "name", "parent_id", "parent_name", "order_index", "created_at", "updated_at", "deleted_at") VALUES 
(1, '系统管理', -1, '', 100, NULL, NULL, NULL),
(2, '用户设置', 1, '系统管理', 10, NULL, NULL, NULL),
(3, '角色设置', 1, '系统管理', 30, NULL, NULL, NULL);