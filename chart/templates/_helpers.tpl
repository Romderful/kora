{{/*
Return the proper Kora image name.
*/}}
{{- define "kora.image" -}}
{{- $imageRoot := dict "registry" .Values.image.registry "repository" .Values.image.repository "tag" (.Values.image.tag | default (printf "v%s" .Chart.AppVersion)) "digest" .Values.image.digest -}}
{{ include "common.images.image" (dict "imageRoot" $imageRoot "global" .Values.global) }}
{{- end -}}

{{/*
Return the proper Docker Image Registry Secret Names.
*/}}
{{- define "kora.imagePullSecrets" -}}
{{- include "common.images.renderPullSecrets" (dict "images" (list .Values.image) "context" $) -}}
{{- end -}}

{{/*
Create the name of the service account to use.
*/}}
{{- define "kora.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
    {{ default (include "common.names.fullname" .) .Values.serviceAccount.name }}
{{- else -}}
    {{ default "default" .Values.serviceAccount.name }}
{{- end -}}
{{- end -}}

{{/*
Build DATABASE_URL from components.
Priority: url > host/port/user/password/name.
*/}}
{{- define "kora.databaseUrl" -}}
{{- if .Values.database.url }}
{{- .Values.database.url }}
{{- else }}
{{- printf "postgres://%s:%s@%s:%v/%s" .Values.database.user .Values.database.password .Values.database.host (int .Values.database.port) .Values.database.name }}
{{- end }}
{{- end }}

{{/*
Return the name of the database secret.
*/}}
{{- define "kora.databaseSecretName" -}}
{{- if .Values.database.existingSecret }}
{{- .Values.database.existingSecret }}
{{- else }}
{{- include "common.names.fullname" . }}-db
{{- end }}
{{- end }}

{{/*
Validate values — fail early on misconfiguration.
*/}}
{{- define "kora.validateValues" -}}
{{- if and (not .Values.database.host) (not .Values.database.url) (not .Values.database.existingSecret) }}
{{- fail "database: you must set one of database.host, database.url, or database.existingSecret" }}
{{- end }}
{{- if and .Values.database.host (not .Values.database.url) (not .Values.database.existingSecret) (not .Values.database.password) }}
{{- fail "database: database.password is required when using database.host (use database.existingSecret for production)" }}
{{- end }}
{{- end }}
